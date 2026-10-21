#!/bin/sh
# Pembungkus Gradle ringkas: cukup menjalankan gradle-wrapper.jar dengan Java.
# (Skrip resmi Gradle jauh lebih panjang; versi ringkas ini melakukan hal yang
#  sama untuk pemakaian normal, tanpa penanganan JVM option khusus.)

APP_HOME=$(cd "$(dirname "$0")" && pwd)
WRAPPER_JAR="$APP_HOME/gradle/wrapper/gradle-wrapper.jar"

if [ ! -f "$WRAPPER_JAR" ]; then
  echo "gradle-wrapper.jar tidak ditemukan di $WRAPPER_JAR" >&2
  exit 1
fi

if [ -n "$JAVA_HOME" ] && [ -x "$JAVA_HOME/bin/java" ]; then
  JAVA_CMD="$JAVA_HOME/bin/java"
else
  JAVA_CMD=java
fi

exec "$JAVA_CMD" -classpath "$WRAPPER_JAR" org.gradle.wrapper.GradleWrapperMain "$@"
