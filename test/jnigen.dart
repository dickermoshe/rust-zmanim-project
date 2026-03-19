import 'dart:io';

import 'package:jnigen/jnigen.dart';

void main(List<String> args) {
  final packageRoot = Platform.script;

  generateJniBindings(
    Config(
      outputConfig: OutputConfig(
        dartConfig: DartCodeOutputConfig(
          // Required. Output path for generated bindings.
          path: packageRoot.resolve('lib/src/java/kosher_java.g.dart'),
          // Optional. Write bindings into a single file (instead of one file per class).
          structure: OutputStructure.singleFile,
        ),
      ),
      // Optional. List of directories that contain the source files for which to generate bindings.
      sourcePath: [packageRoot.resolve('../../zmanim-modern/src/main/java')],
      // Required. List of classes or packages for which bindings should be generated.
      classes: [
        // KosherJava
        'com.kosherjava.zmanim.ComprehensiveZmanimCalendar',
        'com.kosherjava.zmanim.ZmanimCalendar',
        'com.kosherjava.zmanim.AstronomicalCalendar',
        'com.kosherjava.zmanim.util.AstronomicalCalculator',
        'com.kosherjava.zmanim.util.GeoLocation',
        'com.kosherjava.zmanim.hebrewcalendar.JewishDate',
        // Java Time
        'java.time.ZonedDateTime',
        'java.time.LocalDateTime',
        'java.time.LocalDate',
        'java.time.Instant',
        'java.time.ZoneId',
      ],
    ),
  );
}
