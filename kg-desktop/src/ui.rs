//! Penggambar jendela (egui).
//!
//! Setiap layar di sini **hanya membaca** data yang sudah disiapkan Rust:
//! `home()`, `QuizStateFfi`, `QuizResultFfi`, `TopicFfi`. Tidak ada aturan
//! permainan, teks label, atau perhitungan skor yang ditulis di file ini —
//! teks tombol pun datang dari Rust (mis. `next_label`, `resume_label`).

use eframe::egui;
use egui::{Align, Color32, FontId, Layout, Margin, RichText, Rounding, Stroke, Vec2};

use kg_ffi::{BlockFfi, HomeFfi, QuizStateFfi, TopicCardFfi};

use crate::app::{App, LatCheck, Route};
use crate::palette::{dims, text_size, Palette};
use crate::TITLE;

/// Buka jendela aplikasi dan jalankan sampai ditutup.
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 900.0])
            .with_min_inner_size([420.0, 520.0])
            .with_title(TITLE),
        ..Default::default()
    };

    let store = crate::store::Store::open_default();
    let speaker = crate::audio::Speaker::detect();

    eframe::run_native(
        TITLE,
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            let app = App::new(store, speaker);
            Ok(Box::new(KgWindow::new(app)))
        }),
    )
}

/// Muat font Korea bila tersedia di sistem, supaya 한글 tidak tampil sebagai
/// kotak kosong. egui memakai font bawaan yang tidak memuat Hangul.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates: &[&str] = if cfg!(windows) {
        &[
            "C:/Windows/Fonts/malgun.ttf",
            "C:/Windows/Fonts/gulim.ttc",
            "C:/Windows/Fonts/batang.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "/Library/Fonts/AppleGothic.ttf",
        ]
    } else {
        &[
            "/usr/share/fonts/truetype/nanum/NanumGothic.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansKR-Regular.otf",
        ]
    };

    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else { continue };
        // .ttc (koleksi) tidak didukung egui; lewati bila bukan ttf/otf.
        if bytes.len() < 4 {
            continue;
        }
        fonts
            .font_data
            .insert("korean".to_owned(), egui::FontData::from_owned(bytes));
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts.families.entry(family).or_default().push("korean".to_owned());
        }
        ctx.set_fonts(fonts);
        return;
    }
}

/// Jendela utama: latar, header, layar aktif, footer.
pub struct KgWindow {
    app: App,
    /// Konfirmasi "Hapus progres?".
    confirm_reset: bool,
    /// Persentase cincin skor yang dianimasikan (0..1).
    ring: f32,
}

impl KgWindow {
    fn new(app: App) -> Self {
        KgWindow { app, confirm_reset: false, ring: 0.0 }
    }

    fn sync_ring(&mut self, ctx: &egui::Context, target: f32) {
        if (self.ring - target).abs() < 0.002 {
            self.ring = target;
            return;
        }
        self.ring += (target - self.ring) * 0.18;
        ctx.request_repaint();
    }
}

impl eframe::App for KgWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let c = self.app.palette();
        apply_style(ctx, &c);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(c.bg))
            .show(ctx, |ui| {
                self.header(ui, &c);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Kolom tengah selebar 720px seperti `.app` di CSS.
                        let avail = ui.available_width();
                        let width = avail.min(dims::MAX_WIDTH);
                        let pad = ((avail - width) / 2.0).max(0.0);
                        ui.horizontal(|ui| {
                            ui.add_space(pad);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                self.body(ui, &c);
                                ui.add_space(dims::BOTTOM_PADDING);
                            });
                        });
                    });
            });

        self.reset_dialog(ctx, &c);
        self.toast(ctx, &c);
    }
}

impl KgWindow {
    /* ----------------------------------------------------------- header */

    fn header(&mut self, ui: &mut egui::Ui, c: &Palette) {
        egui::Frame::none()
            .fill(c.bg_soft)
            .inner_margin(Margin::symmetric(18.0f32, 10.0f32))
            .stroke(Stroke::new(1.0f32, c.line))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let home = ui.add(
                        egui::Label::new(
                            RichText::new("한국어 문법")
                                .size(17.0)
                                .strong()
                                .color(c.text),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if home.clicked() && self.app.route != Route::Home {
                        self.app.go_home();
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new("Korean Grammar").size(13.0f32).color(c.text_faint));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let icon = if c.is_dark { "☀" } else { "☾" };
                        if ui
                            .add(egui::Button::new(RichText::new(icon).size(16.0f32).color(c.text_dim)).frame(false))
                            .on_hover_text(if c.is_dark { "Mode terang" } else { "Mode gelap" })
                            .clicked()
                        {
                            self.app.toggle_theme();
                        }
                    });
                });
            });
    }

    /* ------------------------------------------------------------- body */

    fn body(&mut self, ui: &mut egui::Ui, c: &Palette) {
        ui.add_space(18.0);
        match self.app.route.clone() {
            Route::Home => self.home(ui, c),
            Route::Lesson { topic_id } => self.lesson(ui, c, &topic_id),
            Route::Latihan { topic_id } => self.latihan(ui, c, &topic_id),
            Route::Quiz { .. } => self.quiz(ui, c),
            Route::Result { topic_id, from } => self.result(ui, c, &topic_id, &from),
        }
    }

    /* ------------------------------------------------------------- home */

    fn home(&mut self, ui: &mut egui::Ui, c: &Palette) {
        let home: HomeFfi = self.app.home();

        card(ui, c, 22.0, |ui| {
            ui.label(RichText::new("안녕하세요! Mulai dari sini").size(14.0f32).strong().color(c.accent));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Grammar Korea untuk pemula")
                    .size(text_size::HERO_TITLE)
                    .strong()
                    .color(c.text),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "10 topik dari nol sampai bisa membuat kalimat sendiri. \
                     Baca materinya, lalu uji dengan kuis pilihan ganda.",
                )
                .size(text_size::BODY)
                .color(c.text_dim),
            );

            ui.add_space(18.0);
            let w = ui.available_width();
            ui.horizontal(|ui| {
                let gap = 10.0;
                let each = ((w - gap * 2.0) / 3.0).max(60.0);
                mini_stat(ui, c, &home.topics_done, "topik tuntas", each);
                ui.add_space(gap);
                mini_stat(ui, c, &home.quiz_score, "rata-rata kuis", each);
                ui.add_space(gap);
                mini_stat(ui, c, &home.answered, "soal dijawab", each);
            });

            ui.add_space(18.0);
            let full = ui.available_width();
            if primary_button(ui, c, &home.resume_label, full).clicked() {
                let id = home.resume_topic.clone();
                self.app.open_topic(&id);
            }
            if home.has_progress {
                ui.add_space(9.0);
                if secondary_button(ui, c, "Hapus progres", full).clicked() {
                    self.confirm_reset = true;
                }
            }
        });

        ui.add_space(24.0);
        ui.label(RichText::new("DAFTAR MATERI").size(text_size::TINY).strong().color(c.text_faint));
        ui.add_space(10.0);

        let cards: Vec<TopicCardFfi> = home.cards.clone();
        for card_data in cards {
            if topic_row(ui, c, &card_data) {
                self.app.open_topic(&card_data.id);
            }
            ui.add_space(9.0);
        }

        ui.add_space(18.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(
                    "Progresmu disimpan otomatis di komputer ini.\n\
                     Semua materi sudah ada di dalam aplikasi, jadi bisa dibuka tanpa internet.",
                )
                .size(text_size::TINY)
                .color(c.text_faint),
            );
        });
    }

    /* ----------------------------------------------------------- lesson */

    fn lesson(&mut self, ui: &mut egui::Ui, c: &Palette, topic_id: &str) {
        let Some(topic) = self
            .app
            .topics
            .iter()
            .find(|t| t.id == topic_id)
            .cloned()
        else {
            self.app.go_home();
            return;
        };

        if back_link(ui, c, "‹ Kembali ke daftar") {
            self.app.go_home();
        }
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            topic_icon(ui, c, &topic.icon, 54.0, 24.0, false);
            ui.add_space(14.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(topic.meta.to_uppercase()).size(text_size::TINY).strong().color(c.accent));
                ui.label(
                    RichText::new(&topic.title)
                        .size(text_size::LESSON_TITLE)
                        .strong()
                        .color(c.text),
                );
                ui.label(RichText::new(&topic.title_id).size(14.0f32).color(c.text_dim));
            });
        });

        ui.add_space(16.0);
        // `.lesson-intro`: latar aksen lembut + garis aksen di kiri.
        egui::Frame::none()
            .fill(c.accent_soft)
            .rounding(Rounding {
                nw: 0.0,
                ne: dims::RADIUS_SM,
                sw: 0.0,
                se: dims::RADIUS_SM,
            })
            .inner_margin(Margin::symmetric(14.0f32, 14.0f32))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 52.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, c.accent);
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&topic.intro).size(14.5f32).color(c.text));
                    });
                });
            });

        ui.add_space(20.0);
        let tts = self.app.speaker.available();
        for block in &topic.blocks {
            // Tombol 🔊 mengembalikan teks Korea yang harus dibacakan.
            if let Some(ko) = block_view(ui, c, block, tts) {
                self.app.speak(&ko);
            }
            ui.add_space(14.0);
        }

        ui.add_space(16.0);
        let full = ui.available_width();
        if primary_button(ui, c, &topic.quiz_label, full).clicked() {
            self.app.start_quiz(topic_id, "lesson");
        }
        ui.add_space(9.0);
        if secondary_button(ui, c, "Latihan menulis", full).clicked() {
            self.app.open_latihan(topic_id);
        }

        let (prev, next) = self.app.neighbours(topic_id);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let half = ((ui.available_width() - 10.0) / 2.0).max(80.0);
            if let Some(p) = prev {
                if secondary_button(ui, c, &format!("‹ {p}"), half).clicked() {
                    self.app.open_prev_topic(topic_id);
                }
                ui.add_space(10.0);
            }
            if let Some(n) = next {
                if secondary_button(ui, c, &format!("{n} ›"), half).clicked() {
                    self.app.open_next_topic(topic_id);
                }
            }
        });
    }

    /* ---------------------------------------------------------- latihan */

    fn latihan(&mut self, ui: &mut egui::Ui, c: &Palette, topic_id: &str) {
        let Some(topic) = self.app.topics.iter().find(|t| t.id == topic_id).cloned() else {
            self.app.go_home();
            return;
        };

        let back = format!("‹ Kembali ke {}", topic.title_id);
        if back_link(ui, c, &back) {
            self.app.open_topic(topic_id);
        }
        ui.add_space(12.0);

        let mut edits: Vec<(usize, String)> = Vec::new();
        let mut do_reveal = false;
        let mut do_clear = false;

        card(ui, c, 22.0, |ui| {
            ui.label(RichText::new("LATIHAN MENULIS").size(text_size::TINY).strong().color(c.accent));
            ui.add_space(2.0);
            ui.label(
                RichText::new("Tulis dalam bahasa Korea")
                    .size(text_size::LESSON_TITLE)
                    .strong()
                    .color(c.text),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Ketikan atau tempel kalimatnya. Cocokkan tanpa melihat romanisasi.")
                    .size(14.0)
                    .color(c.text_dim),
            );

            ui.add_space(18.0);
            for (i, item) in topic.lat.iter().enumerate() {
                let entry = self.app.lat_entries.get(i).cloned().unwrap_or_default();
                let border = match entry.check {
                    LatCheck::Ok => c.ok,
                    LatCheck::Bad => c.bad,
                    LatCheck::Empty => c.line,
                };
                egui::Frame::none()
                    .fill(c.surface2)
                    .stroke(Stroke::new(1.0f32, border))
                    .rounding(dims::RADIUS)
                    .inner_margin(Margin::symmetric(15.0f32, 13.0f32))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new(format!("Artinya: {}", item.id)).size(13.5).color(c.text_dim));
                        ui.add_space(6.0);
                        let mut text = entry.typed.clone();
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut text)
                                .hint_text("Tulis kalimat Korea…")
                                .desired_width(f32::INFINITY)
                                .font(FontId::proportional(16.5f32))
                                .margin(Margin::symmetric(10.0f32, 8.0f32)),
                        );
                        if resp.changed() {
                            edits.push((i, text));
                        }
                        if !entry.inline.is_empty() {
                            ui.add_space(5.0);
                            let col = if entry.check == LatCheck::Ok { c.ok } else { c.bad };
                            ui.label(RichText::new(&entry.inline).size(13.0f32).strong().color(col));
                        }
                    });
                ui.add_space(12.0);
            }

            if !self.app.lat_status.is_empty() {
                ui.label(RichText::new(&self.app.lat_status).size(13.5f32).color(c.text_dim));
                ui.add_space(8.0);
            }

            let half = ((ui.available_width() - 10.0) / 2.0).max(80.0);
            ui.horizontal(|ui| {
                if secondary_button(ui, c, "Tampilkan jawaban", half).clicked() {
                    do_reveal = true;
                }
                ui.add_space(10.0);
                if secondary_button(ui, c, "Kosongkan", half).clicked() {
                    do_clear = true;
                }
            });
        });

        for (i, text) in edits {
            self.app.type_latihan(topic_id, i, text);
        }
        if do_reveal {
            self.app.reveal_latihan(topic_id);
        }
        if do_clear {
            self.app.clear_latihan();
        }
    }

    /* ------------------------------------------------------------- quiz */

    fn quiz(&mut self, ui: &mut egui::Ui, c: &Palette) {
        let Some(state) = self.app.quiz.clone().or_else(kg_ffi::quiz_state) else {
            self.app.go_home();
            return;
        };
        self.app.quiz = Some(state.clone());

        let mut pick: Option<i32> = None;
        let mut next = false;
        let mut close = false;

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(RichText::new("✕").size(15.0f32).color(c.text_dim)).frame(false)).clicked() {
                close = true;
            }
            ui.add_space(10.0);
            let pill_w = 74.0;
            let label_w = (ui.available_width() - pill_w - 10.0).max(60.0);
            ui.allocate_ui(Vec2::new(label_w, 20.0), |ui| {
                ui.label(
                    RichText::new(&state.topic_label)
                        .size(14.0)
                        .strong()
                        .color(c.text_dim),
                );
            });
            score_pill(ui, c, &state.score_label);
        });

        ui.add_space(10.0);
        progress_bar(ui, c, state.progress as f32 / 100.0);
        ui.add_space(14.0);

        card(ui, c, 18.0, |ui| {
            ui.label(RichText::new(state.counter.to_uppercase()).size(text_size::TINY).strong().color(c.text_faint));
            ui.add_space(8.0);
            ui.label(
                RichText::new(&state.question)
                    .size(text_size::QUIZ_QUESTION)
                    .strong()
                    .color(c.text),
            );
            ui.add_space(16.0);

            for (i, text) in state.options.iter().enumerate() {
                let letter = state.letters.get(i).cloned().unwrap_or_default();
                if option_row(ui, c, &state, i, &letter, text) {
                    pick = Some(i as i32);
                }
                ui.add_space(8.0);
            }

            if state.answered && !state.explanation.is_empty() {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(c.surface2)
                    .rounding(dims::RADIUS_SM)
                    .inner_margin(Margin::symmetric(14.0f32, 12.0f32))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_top(|ui| {
                            let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 34.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, c.accent);
                            ui.add_space(11.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&state.explanation).size(14.0f32).color(c.text_dim));
                            });
                        });
                    });
            }

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                let feedback_col = match state.feedback_kind.as_str() {
                    "ok" => c.ok,
                    "bad" => c.bad,
                    _ => c.text_dim,
                };
                let btn_w = if state.answered { 120.0 } else { 0.0 };
                let text_w = (ui.available_width() - btn_w - 10.0).max(60.0);
                ui.allocate_ui(Vec2::new(text_w, 24.0), |ui| {
                    ui.label(
                        RichText::new(&state.feedback)
                            .size(14.0)
                            .strong()
                            .color(feedback_col),
                    );
                });
                if state.answered && primary_button(ui, c, &state.next_label, btn_w).clicked() {
                    next = true;
                }
            });
        });

        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Klik pilihan A–D untuk menjawab")
                    .size(text_size::TINY)
                    .color(c.text_faint),
            );
        });

        if close {
            self.app.quit_quiz();
            return;
        }
        if let Some(i) = pick {
            self.app.pick(i);
        } else if next {
            self.app.advance();
        }
    }

    /* ----------------------------------------------------------- result */

    fn result(&mut self, ui: &mut egui::Ui, c: &Palette, topic_id: &str, from: &str) {
        let Some(res) = self.app.result.clone() else {
            self.app.go_home();
            return;
        };
        let target = res.percent.min(100) as f32 / 100.0;
        self.sync_ring(ui.ctx(), target);

        let ring_color = match res.ring_tone.as_str() {
            "ok" => c.ok,
            "bad" => c.bad,
            _ => c.accent,
        };

        let mut retry = false;
        let mut back_lesson = false;
        let mut next_topic = false;

        card(ui, c, 22.0, |ui| {
            ui.vertical_centered(|ui| {
                // Cincin skor (menggantikan animasi stroke-dashoffset di CSS).
                let size = Vec2::splat(132.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let painter = ui.painter();
                let stroke = 9.0;
                painter.circle_stroke(rect.center(), rect.width() / 2.0 - stroke / 2.0, Stroke::new(stroke, c.surface2));
                let n = (self.ring * 64.0).round() as usize;
                for i in 0..n {
                    let a0 = std::f32::consts::TAU * (i as f32 / 64.0) - std::f32::consts::FRAC_PI_2;
                    let a1 = std::f32::consts::TAU * ((i + 1) as f32 / 64.0) - std::f32::consts::FRAC_PI_2;
                    let r = rect.width() / 2.0 - stroke / 2.0;
                    let p0 = rect.center() + Vec2::new(a0.cos(), a0.sin()) * r;
                    let p1 = rect.center() + Vec2::new(a1.cos(), a1.sin()) * r;
                    painter.line_segment([p0, p1], Stroke::new(stroke, ring_color));
                }

                // Angka di tengah cincin
                let center = rect.center();
                painter.text(
                    center - Vec2::new(0.0, 6.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}%", res.percent),
                    FontId::proportional(28.0f32),
                    c.text,
                );
                painter.text(
                    center + Vec2::new(0.0, 16.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{} / {}", res.correct, res.total),
                    FontId::proportional(12.0f32),
                    c.text_faint,
                );

                ui.add_space(16.0);
                ui.label(RichText::new(&res.headline).size(21.0f32).strong().color(c.text));
                ui.add_space(6.0);
                ui.label(RichText::new(&res.message).size(14.5f32).color(c.text_dim));

                ui.add_space(20.0);
                let w = ui.available_width();
                ui.horizontal(|ui| {
                    let gap = 10.0;
                    let each = ((w - gap * 2.0) / 3.0).max(60.0);
                    result_stat(ui, c, &res.correct.to_string(), "Benar", c.ok, each);
                    ui.add_space(gap);
                    result_stat(ui, c, &res.wrong.to_string(), "Salah", c.bad, each);
                    ui.add_space(gap);
                    result_stat(ui, c, &res.time, "Waktu", c.text, each);
                });

                ui.add_space(18.0);
                let full = ui.available_width();
                if primary_button(ui, c, "Ulangi kuis", full).clicked() {
                    retry = true;
                }
                ui.add_space(9.0);
                if secondary_button(ui, c, "Baca materi lagi", full).clicked() {
                    back_lesson = true;
                }
                let has_next = self
                    .app
                    .topics
                    .iter()
                    .position(|t| t.id == topic_id)
                    .map(|i| i + 1 < self.app.topics.len())
                    .unwrap_or(false);
                if has_next {
                    ui.add_space(9.0);
                    if secondary_button(ui, c, "Topik berikutnya", full).clicked() {
                        next_topic = true;
                    }
                }
            });
        });

        if !res.wrong_answers.is_empty() {
            ui.add_space(12.0);
            card(ui, c, 20.0, |ui| {
                ui.label(
                    RichText::new("Pembahasan jawaban yang belum tepat")
                        .size(15.0)
                        .strong()
                        .color(c.text),
                );
                ui.add_space(10.0);
                for w in &res.wrong_answers {
                    egui::Frame::none()
                        .fill(c.surface2)
                        .stroke(Stroke::new(1.0f32, c.line))
                        .rounding(dims::RADIUS_SM)
                        .inner_margin(Margin::symmetric(14.0f32, 12.0f32))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(RichText::new(&w.q).size(14.0f32).strong().color(c.text));
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("Jawabanmu: ").size(13.5f32).color(c.text_dim));
                                ui.label(RichText::new(&w.picked).size(13.5f32).strong().color(c.bad));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("Yang benar: ").size(13.5f32).color(c.text_dim));
                                ui.label(RichText::new(&w.right).size(13.5f32).strong().color(c.ok));
                            });
                            if !w.why.is_empty() {
                                ui.add_space(5.0);
                                ui.label(RichText::new(&w.why).size(13.0f32).color(c.text_faint));
                            }
                        });
                    ui.add_space(10.0);
                }
            });
        }

        if retry {
            self.app.start_quiz(topic_id, from);
        } else if back_lesson {
            self.app.open_topic(topic_id);
        } else if next_topic {
            self.app.open_next_topic(topic_id);
        }
    }

    /* ---------------------------------------------------------- dialog */

    fn reset_dialog(&mut self, ctx: &egui::Context, c: &Palette) {
        if !self.confirm_reset {
            return;
        }
        let mut open = true;
        let mut do_reset = false;
        // `open` sedang dipinjam oleh .open(&mut open), jadi tombol Batal
        // menandai niat lewat flag terpisah.
        let mut cancel = false;
        egui::Window::new("Hapus progres?")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(egui::Frame::none().fill(c.surface).stroke(Stroke::new(1.0f32, c.line)).inner_margin(Margin::same(20.0)))
            .show(ctx, |ui| {
                ui.label(RichText::new("Hapus semua progres belajar di komputer ini?").color(c.text_dim));
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if danger_button(ui, c, "Hapus", 90.0).clicked() {
                        do_reset = true;
                    }
                    ui.add_space(8.0);
                    if secondary_button(ui, c, "Batal", 90.0).clicked() {
                        cancel = true;
                    }
                });
            });
        if do_reset {
            self.app.reset_progress();
            self.confirm_reset = false;
        } else if !open || cancel {
            self.confirm_reset = false;
        }
    }

    fn toast(&mut self, ctx: &egui::Context, c: &Palette) {
        let Some(msg) = self.app.toast.clone() else { return };
        egui::Area::new(egui::Id::new("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0])
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(c.surface2)
                    .stroke(Stroke::new(1.0f32, c.bad))
                    .rounding(dims::RADIUS_SM)
                    .inner_margin(Margin::symmetric(14.0f32, 10.0f32))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&msg).size(13.0f32).color(c.text));
                            ui.add_space(8.0);
                            if ui.add(egui::Button::new(RichText::new("✕").size(12.0f32).color(c.text_faint)).frame(false)).clicked() {
                                self.app.toast = None;
                            }
                        });
                    });
            });
    }
}

/* ====================================================== komponen kecil == */

fn apply_style(ctx: &egui::Context, c: &Palette) {
    let mut style = (*ctx.style()).clone();
    style.visuals.override_text_color = Some(c.text);
    style.visuals.panel_fill = c.bg;
    style.visuals.window_fill = c.surface;
    style.visuals.extreme_bg_color = c.surface;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0f32, c.line);
    style.visuals.selection.bg_fill = c.accent;
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(14.0, 9.0);
    ctx.set_style(style);
}

fn card(ui: &mut egui::Ui, c: &Palette, pad: f32, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(c.surface)
        .stroke(Stroke::new(1.0f32, c.line))
        .rounding(dims::RADIUS_LG)
        .inner_margin(Margin::same(pad))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn mini_stat(ui: &mut egui::Ui, c: &Palette, value: &str, label: &str, width: f32) {
    egui::Frame::none()
        .fill(c.surface2)
        .stroke(Stroke::new(1.0f32, c.line))
        .rounding(dims::RADIUS_SM)
        .inner_margin(Margin::symmetric(10.0f32, 10.0f32))
        .show(ui, |ui| {
            ui.set_width(width - 22.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(value).size(18.0f32).strong().color(c.text));
                ui.label(RichText::new(label).size(11.0f32).color(c.text_faint));
            });
        });
}

fn result_stat(ui: &mut egui::Ui, c: &Palette, value: &str, label: &str, col: Color32, width: f32) {
    egui::Frame::none()
        .fill(c.surface2)
        .stroke(Stroke::new(1.0f32, c.line))
        .rounding(dims::RADIUS_SM)
        .inner_margin(Margin::symmetric(10.0f32, 10.0f32))
        .show(ui, |ui| {
            ui.set_width(width - 22.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(value).size(20.0f32).strong().color(col));
                ui.label(RichText::new(label).size(11.0f32).color(c.text_faint));
            });
        });
}

/// Tombol utama (aksen).
fn primary_button(ui: &mut egui::Ui, c: &Palette, label: &str, width: f32) -> egui::Response {
    let text = RichText::new(label).size(14.5f32).strong().color(Color32::WHITE);
    ui.add_sized(
        Vec2::new(width, dims::BTN_MIN_HEIGHT),
        egui::Button::new(text)
            .fill(c.accent)
            .stroke(Stroke::NONE)
            .rounding(dims::RADIUS_SM),
    )
}

/// Tombol kedua (garis tepi).
fn secondary_button(ui: &mut egui::Ui, c: &Palette, label: &str, width: f32) -> egui::Response {
    let text = RichText::new(label).size(13.5f32).color(c.text);
    ui.add_sized(
        Vec2::new(width, dims::BTN_MIN_HEIGHT),
        egui::Button::new(text)
            .fill(c.surface2)
            .stroke(Stroke::new(1.0f32, c.line))
            .rounding(dims::RADIUS_SM),
    )
}

fn danger_button(ui: &mut egui::Ui, c: &Palette, label: &str, width: f32) -> egui::Response {
    let text = RichText::new(label).size(13.5f32).strong().color(Color32::WHITE);
    ui.add_sized(
        Vec2::new(width, dims::BTN_MIN_HEIGHT),
        egui::Button::new(text)
            .fill(c.bad)
            .stroke(Stroke::NONE)
            .rounding(dims::RADIUS_SM),
    )
}

fn back_link(ui: &mut egui::Ui, c: &Palette, label: &str) -> bool {
    let r = ui.add(
        egui::Label::new(RichText::new(label).size(13.5f32).color(c.text_dim))
            .sense(egui::Sense::click()),
    );
    r.clicked()
}

fn score_pill(ui: &mut egui::Ui, c: &Palette, label: &str) {
    egui::Frame::none()
        .fill(c.accent_soft)
        .rounding(999.0)
        .inner_margin(Margin::symmetric(10.0f32, 4.0f32))
        .show(ui, |ui| {
            ui.label(RichText::new(label).size(12.0f32).strong().color(c.accent));
        });
}

fn progress_bar(ui: &mut egui::Ui, c: &Palette, frac: f32) {
    let h = 6.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), h), egui::Sense::hover());
    ui.painter().rect_filled(rect, 3.0, c.surface2);
    let w = rect.width() * frac.clamp(0.0, 1.0);
    if w > 0.0 {
        let bar = egui::Rect::from_min_size(rect.min, Vec2::new(w, h));
        ui.painter().rect_filled(bar, 3.0, c.accent);
    }
}

fn topic_icon(ui: &mut egui::Ui, c: &Palette, icon: &str, size: f32, font: f32, done: bool) {
    let bg = if done { c.ok_soft } else { c.surface2 };
    let border = if done { c.ok } else { c.line };
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    ui.painter().rect_filled(rect, dims::RADIUS_SM, bg);
    ui.painter().rect_stroke(
        rect,
        dims::RADIUS_SM,
        Stroke::new(1.0f32, border),
        );
    ui.painter()
        .text(rect.center(), egui::Align2::CENTER_CENTER, icon, FontId::proportional(font), c.text);
}

/// Satu kartu topik di daftar; `true` bila diklik.
fn topic_row(ui: &mut egui::Ui, c: &Palette, card_data: &TopicCardFfi) -> bool {
    let mut clicked = false;
    let resp = egui::Frame::none()
        .fill(c.surface)
        .stroke(Stroke::new(1.0f32, c.line))
        .rounding(dims::RADIUS)
        .inner_margin(Margin::symmetric(15.0f32, 13.0f32))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                topic_icon(ui, c, &card_data.icon, 40.0, 19.0, card_data.done);
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&card_data.name).size(15.0f32).strong().color(c.text));
                    ui.label(RichText::new(&card_data.title_id).size(12.5f32).color(c.text_dim));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (bg, fg) = if card_data.badge_is_score {
                        (c.ok_soft, c.ok)
                    } else {
                        (c.surface2, c.text_dim)
                    };
                    egui::Frame::none()
                        .fill(bg)
                        .rounding(999.0)
                        .inner_margin(Margin::symmetric(9.0f32, 3.0f32))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&card_data.badge).size(11.5f32).strong().color(fg));
                        });
                });
            });
        })
        .response;

    let resp = resp.interact(egui::Sense::click());
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        clicked = true;
    }
    clicked
}

/// Gambar satu blok materi. Mengembalikan teks Korea yang harus dibacakan
/// bila tombol 🔊 di salah satu contoh ditekan.
fn block_view(ui: &mut egui::Ui, c: &Palette, block: &BlockFfi, tts: bool) -> Option<String> {
    let mut speak: Option<String> = None;
    match block.kind.as_str() {
        "heading" => {
            ui.label(RichText::new(&block.text).size(16.0f32).strong().color(c.text));
            ui.add_space(6.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, c.line);
        }
        "text" => {
            ui.label(RichText::new(&block.text).size(text_size::BODY).color(c.text_dim));
        }
        "formula" => {
            egui::Frame::none()
                .fill(c.surface)
                .stroke(Stroke::new(1.0f32, c.line))
                .rounding(dims::RADIUS)
                .inner_margin(Margin::symmetric(15.0f32, 13.0f32))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        for (i, part) in block.parts.iter().enumerate() {
                            if i > 0 {
                                ui.label(RichText::new("+").strong().color(c.text_faint));
                            }
                            egui::Frame::none()
                                .fill(c.surface2)
                                .stroke(Stroke::new(1.0f32, c.line))
                                .rounding(10.0)
                                .inner_margin(Margin::symmetric(12.0f32, 6.0f32))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(part).size(14.0f32).strong().color(c.text));
                                });
                        }
                    });
                });
        }
        "rules" => {
            egui::Frame::none()
                .fill(c.surface)
                .stroke(Stroke::new(1.0f32, c.line))
                .rounding(dims::RADIUS)
                .inner_margin(Margin::same(0.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for (i, row) in block.rules.iter().enumerate() {
                        if i > 0 {
                            let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
                            ui.painter().rect_filled(r, 0.0, c.line);
                        }
                        egui::Frame::none().inner_margin(Margin::symmetric(13.0f32, 10.0f32)).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let w = ui.available_width();
                            ui.horizontal(|ui| {
                                ui.allocate_ui(Vec2::new(w * 0.30, 20.0), |ui| {
                                    ui.label(RichText::new(&row.a).size(13.5f32).strong().color(c.text));
                                });
                                ui.allocate_ui(Vec2::new(w * 0.28, 20.0), |ui| {
                                    ui.label(RichText::new(&row.b).size(13.5f32).color(c.text_dim));
                                });
                                ui.allocate_ui(Vec2::new(w * 0.36, 20.0), |ui| {
                                    ui.label(RichText::new(&row.c).size(13.5f32).color(c.text_dim));
                                });
                            });
                        });
                    }
                });
        }
        "examples" => {
            for ex in &block.examples {
                egui::Frame::none()
                    .fill(c.surface)
                    .stroke(Stroke::new(1.0f32, c.line))
                    .rounding(dims::RADIUS)
                    .inner_margin(Margin::symmetric(15.0f32, 13.0f32))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_top(|ui| {
                            let btn_w = if tts { 34.0 } else { 0.0 };
                            let text_w = (ui.available_width() - btn_w - 8.0).max(60.0);
                            ui.allocate_ui(Vec2::new(text_w, 26.0), |ui| {
                                ui.label(
                                    RichText::new(&ex.ko)
                                        .size(text_size::EXAMPLE_KO)
                                        .strong()
                                        .color(c.text),
                                );
                            });
                            if tts
                                && ui
                                    .add(egui::Button::new(RichText::new("🔊").size(14.0f32)).frame(false))
                                    .on_hover_text("Bacakan")
                                    .clicked()
                            {
                                speak = Some(ex.ko.clone());
                            }
                        });
                        if !ex.rom.is_empty() {
                            ui.label(RichText::new(&ex.rom).size(13.0f32).italics().color(c.text_faint));
                        }
                        if !ex.id.is_empty() {
                            ui.add_space(3.0);
                            ui.label(RichText::new(&ex.id).size(14.5f32).color(c.text_dim));
                        }
                        if !ex.why.is_empty() {
                            ui.add_space(5.0);
                            ui.label(RichText::new(&ex.why).size(13.0f32).color(c.accent));
                        }
                    });
                ui.add_space(9.0);
            }
        }
        "note" => {
            egui::Frame::none()
                .fill(c.surface2)
                .stroke(Stroke::new(1.0f32, c.line))
                .rounding(dims::RADIUS)
                .inner_margin(Margin::symmetric(14.0f32, 12.0f32))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(RichText::new("Catatan: ").size(14.0f32).strong().color(c.warn));
                        ui.label(RichText::new(&block.text).size(14.0f32).color(c.text_dim));
                    });
                });
        }
        _ => {
            ui.label(RichText::new(&block.text).size(text_size::BODY).color(c.text_dim));
        }
    }
    speak
}

/// Satu pilihan jawaban; `true` bila diklik.
fn option_row(
    ui: &mut egui::Ui,
    c: &Palette,
    state: &QuizStateFfi,
    index: usize,
    letter: &str,
    text: &str,
) -> bool {
    let is_right = state.answered && index == state.answer as usize;
    let is_picked_wrong = state.answered && index == state.picked as usize && !is_right;
    let dimmed = state.answered && !is_right && !is_picked_wrong;

    let border = if is_right {
        c.ok
    } else if is_picked_wrong {
        c.bad
    } else {
        c.line
    };
    let bg = if is_right {
        c.ok_soft
    } else if is_picked_wrong {
        c.bad_soft
    } else {
        c.surface2
    };

    let resp = egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.5f32, border))
        .rounding(dims::RADIUS_SM)
        .inner_margin(Margin::symmetric(14.0f32, 12.0f32))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                let lbg = if is_right {
                    c.ok
                } else if is_picked_wrong {
                    c.bad
                } else {
                    c.surface
                };
                ui.painter().rect_filled(rect, 8.0, lbg);
                ui.painter().rect_stroke(
                    rect,
                    8.0,
                    Stroke::new(1.0f32, if is_right || is_picked_wrong { lbg } else { c.line }),
                    );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    letter,
                    FontId::proportional(11.5f32),
                    if is_right || is_picked_wrong { Color32::WHITE } else { c.text_faint },
                );
                ui.add_space(10.0);
                let col = if dimmed { c.text_dim } else { c.text };
                ui.label(RichText::new(text).size(15.0f32).color(col));
            });
        })
        .response;

    let resp = resp.interact(egui::Sense::click());
    if !state.answered && resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    !state.answered && resp.clicked()
}
