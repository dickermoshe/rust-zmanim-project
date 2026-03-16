import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:test_with_java/rnd.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:test_with_java/kosher_java.g.dart';
import 'package:test_with_java/src/rust/api.dart';
import 'package:test_with_java/src/rust/frb_generated.dart';
import 'package:jni/jni.dart';

const TEST_ITERATIONS =
    const int.fromEnvironment('TEST_ITERATIONS', defaultValue: 10);

late Function(String) debug;
Future<void> main(List<String> args) async {
  if (args.contains('--debug')) {
    debug = (String message) {
      print(message);
    };
  } else {
    debug = (String message) {};
  }

  // Initialize Rust library
  await RustLib.init(
      externalLibrary: await loadExternalLibrary(ExternalLibraryLoaderConfig(
    stem: 'test_with_java',
    ioDirectory: '../target/release/',
    webPrefix: 'pkg/',
  )));

  // Initialize Java runtime
  Jni.spawn(classPath: _resolveJvmClassPath());

  // Get a list of the timezones that are supported by both Java and Rust
  final javaTimezones =
      ZoneId.getAvailableZoneIds()!.map((e) => e!.toDartString()).toSet();
  final rustTimezones = timezones().toSet();
  final validTimezones = javaTimezones.intersection(rustTimezones).toList();

  // Ensure we can round trip a TestCase to and from JSON
  final testCase = TestCase.random(
      zman: presets().first, iteration: 0, validTimezones: validTimezones);
  final testCase2 = TestCase.fromJson(testCase.toJson(), presets());
  assertFail(testCase == testCase2,
      "TestCase: ${testCase.toJson()}, TestCase2: ${testCase2.toJson()}");

  // Loop through each zman and test it
  for (final zman in presets()) {
    testZman(zman, validTimezones);
  }
  print("All tests passed");
  exit(0);
}

void testZman(ZmanimPreset zman, List<String> validTimezones) {
  final zmanName = zman.name();
  bool testedOne = false;
  debug("Testing $zmanName");
  for (var i = 1; i <= TEST_ITERATIONS; i++) {
    final ran = TestCase.random(
        zman: zman, iteration: i, validTimezones: validTimezones);
    final tested = ran.test();
    if (tested) {
      testedOne = true;
    }
  }
  if (!testedOne) {
    // Because this Zman only happens once a year, it is quite uncommon for it to come up.
    if (zmanName.contains("Chametz")) {
      return;
    }
    throw Exception("Failed to test $zmanName");
  }
}

void assertFail(bool condition, String message) {
  if (!condition) {
    throw Exception(message);
  }
}

class TestCase {
  final int iteration;
  final int year;
  final int month;
  final int day;
  final double latitude;
  final double longitude;
  final double elevation;
  final String timezone;
  final ZmanimPreset zman;
  final int ateretTorahSunsetOffsetMinutes;
  final int candleLightingOffsetMinutes;
  final bool useAstronomicalChatzosForOtherZmanim;
  final bool useElevation;

  TestCase(
      {required this.iteration,
      required this.year,
      required this.month,
      required this.day,
      required this.latitude,
      required this.longitude,
      required this.elevation,
      required this.timezone,
      required this.zman,
      required this.ateretTorahSunsetOffsetMinutes,
      required this.candleLightingOffsetMinutes,
      required this.useAstronomicalChatzosForOtherZmanim,
      required this.useElevation});

  String toJson() {
    return jsonEncode({
      'iteration': iteration,
      'year': year,
      'month': month,
      'day': day,
      'latitude': latitude,
      'longitude': longitude,
      'elevation': elevation,
      'timezone': timezone,
      'zman': zman.name(),
      'ateretTorahSunsetOffsetMinutes': ateretTorahSunsetOffsetMinutes,
      'candleLightingOffsetMinutes': candleLightingOffsetMinutes,
      'useAstronomicalChatzosForOtherZmanim':
          useAstronomicalChatzosForOtherZmanim,
      'useElevation': useElevation,
    });
  }

  static TestCase fromMap(
      Map<String, dynamic> data, List<ZmanimPreset> zmanimPresets) {
    return TestCase(
      iteration: data['iteration'],
      year: data['year'],
      month: data['month'],
      day: data['day'],
      latitude: data['latitude'],
      longitude: data['longitude'],
      elevation: data['elevation'],
      timezone: data['timezone'],
      useElevation: data['useElevation'],
      zman: zmanimPresets.firstWhere((e) => e.name() == data['zman']),
      ateretTorahSunsetOffsetMinutes: data['ateretTorahSunsetOffsetMinutes'],
      candleLightingOffsetMinutes: data['candleLightingOffsetMinutes'],
      useAstronomicalChatzosForOtherZmanim:
          data['useAstronomicalChatzosForOtherZmanim'],
    );
  }

  static TestCase fromJson(String json, List<ZmanimPreset> zmanimPresets) {
    return fromMap(jsonDecode(json), zmanimPresets);
  }

  static TestCase random(
      {required ZmanimPreset zman,
      required int iteration,
      required List<String> validTimezones}) {
    final randomDateTime =
        DateTime.fromMillisecondsSinceEpoch(rnd.getInt(YEARS_MS, -YEARS_MS));
    // TODO: Test with random latitude and longitude
    // final randomLatitude = rnd(-90.0, 90.0);
    // final randomLongitude = rnd(-180.0, 180.0);
    final randomLatitude = rnd(-40.0, 40.0);
    final randomLongitude = rnd(-180.0, 180.0);
    final tz =
        findTimezone(longitude: randomLongitude, latitude: randomLatitude);
    if (!validTimezones.contains(tz)) {
      return random(
          zman: zman, iteration: iteration, validTimezones: validTimezones);
    }
    // TODO: Test with random elevation
    // final randomElevation = rnd(0.0, 400.0);
    final randomElevation = 0.0;
    final randomUseElevation = rnd.getBool();

    final randomAteretTorahSunsetOffsetMinutes = rnd.getInt(0, 60);
    final randomCandleLightingOffsetMinutes = rnd.getInt(0, 60);
    final randomUseAstronomicalChatzosForOtherZmanim = rnd.getBool();
    return TestCase(
      iteration: iteration,
      year: randomDateTime.year,
      month: randomDateTime.month,
      day: randomDateTime.day,
      latitude: randomLatitude,
      longitude: randomLongitude,
      elevation: randomElevation,
      timezone: tz,
      zman: zman,
      ateretTorahSunsetOffsetMinutes: randomAteretTorahSunsetOffsetMinutes,
      candleLightingOffsetMinutes: randomCandleLightingOffsetMinutes,
      useAstronomicalChatzosForOtherZmanim:
          randomUseAstronomicalChatzosForOtherZmanim,
      useElevation: randomUseElevation,
    );
  }

  int? calculateJavaZman() {
    final javaZoneId = ZoneId.of$1(timezone.toJString())!;
    final localDate = LocalDate.of$1(year, month, day);
    final location = GeoLocation.new$1(
        "".toJString(), latitude, longitude, elevation, javaZoneId);
    final calendar = ComprehensiveZmanimCalendar.new1(location);
    calendar.setUseElevation(useElevation);
    calendar.setCandleLightingOffset(candleLightingOffsetMinutes.toDouble());
    calendar.setUseAstronomicalChatzosForOtherZmanim(
        useAstronomicalChatzosForOtherZmanim);
    calendar
        .setAteretTorahSunsetOffset(ateretTorahSunsetOffsetMinutes.toDouble());
    calendar.setLocalDate(localDate);
    // Invoke the method using the JNI API
    final methodId = calendar.jClass.instanceMethodId(
      zman.name(),
      r'()Ljava/time/Instant;',
    );
    final result = methodId.call(calendar, $Instant$NullableType$(), []);
    // ignore: invalid_use_of_internal_member
    if (result?.reference.isNull ?? false) {
      return null;
    }

    return result?.toEpochMilli();
  }

  bool test() {
    try {
      final javaZman = calculateJavaZman();
      final rustZman = calculateRustZman();
      // We do not test for situations where one is null and the other is not,
      // more testing is needed for these cases.
      // if (javaZman == null || rustZman == null) {
      //   continue;
      // }
      assertFail(
          (javaZman != null && rustZman != null) ||
              (javaZman == null && rustZman == null),
          "Java: $javaZman, Rust: $rustZman, TestCase: ${toJson()}");
      if (javaZman == null || rustZman == null) {
        return false;
      }
      debug("Java: $javaZman, Rust: $rustZman");
      final difference = javaZman - rustZman;

      assertFail(difference < (HOURS_MS * 24),
          'Difference : ${formatDifference(difference)}. Java: $javaZman, Rust: $rustZman. TestCase: ${toJson()}');
      debug("Success: $iteration / $TEST_ITERATIONS for ${zman.name()}");
    } catch (e) {
      // Log TestCase to stderr
      stderr.writeln(toJson());
      rethrow;
    }
    return true;
  }

  int? calculateRustZman() {
    final rustZman = calculateZman(
      ateretTorahSunsetOffsetMinutes: ateretTorahSunsetOffsetMinutes,
      candleLightingOffsetMinutes: candleLightingOffsetMinutes,
      useAstronomicalChatzosForOtherZmanim:
          useAstronomicalChatzosForOtherZmanim,
      latitude: latitude,
      longitude: longitude,
      elevation: useElevation ? elevation : 0.0,
      timezone: timezone,
      randomYear: year,
      randomMonth: month,
      randomDay: day,
      zman: zman,
    );
    if (rustZman == null) {
      return null;
    }
    final (formattedRustTimestamp, rustTimestampMs) = rustZman;
    return rustTimestampMs;
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TestCase &&
          iteration == other.iteration &&
          year == other.year &&
          month == other.month &&
          day == other.day &&
          latitude == other.latitude &&
          longitude == other.longitude &&
          elevation == other.elevation &&
          timezone == other.timezone &&
          zman.name() == other.zman.name() &&
          ateretTorahSunsetOffsetMinutes ==
              other.ateretTorahSunsetOffsetMinutes &&
          candleLightingOffsetMinutes == other.candleLightingOffsetMinutes &&
          useAstronomicalChatzosForOtherZmanim ==
              other.useAstronomicalChatzosForOtherZmanim &&
          useElevation == other.useElevation;
}

const YEARS = 200;
const YEARS_MS = YEARS * 365 * 24 * 60 * 60 * 1000;

/// Return a list of paths to the .jar files from KosherJava
List<String> _resolveJvmClassPath() {
  final classPathEntries = <String>[];

  // Prefer compiled jars from the Maven build output.
  final modernTargetDir = Directory('../../zmanim-modern/target');
  if (modernTargetDir.existsSync()) {
    final jars = modernTargetDir
        .listSync()
        .whereType<File>()
        .map((e) => e.path)
        .where((path) =>
            path.endsWith('.jar') &&
            !path.endsWith('-sources.jar') &&
            !path.endsWith('-javadoc.jar'))
        .toList()
      ..sort();
    classPathEntries.addAll(jars);

    final classesDir = Directory('${modernTargetDir.path}/classes');
    if (classesDir.existsSync()) {
      classPathEntries.add(classesDir.path);
    }
  }

  // Optional local compiled jar fallback (if copied into ./test).
  final localCompiledJar = File('./zmanim-2.6.0-SNAPSHOT.jar');
  if (localCompiledJar.existsSync()) {
    classPathEntries.add(localCompiledJar.path);
  }

  if (classPathEntries.isEmpty) {
    throw StateError(
      'Could not find compiled zmanim classes. Build Java first (e.g. run Maven package in ../../zmanim-modern).',
    );
  }

  return classPathEntries.toSet().toList();
}

const HOURS_MS = 60 * 60 * 1000;
const MINUTES_MS = 60 * 1000;
const SECONDS_MS = 1000;
// Show as hours if more than 1 hour, minutes if more than 1 minute, seconds if more than 1 second, milliseconds if less than 1 second
String formatDifference(int differenceMs) {
  if (differenceMs > HOURS_MS) {
    return '${differenceMs / HOURS_MS} hours';
  }
  if (differenceMs > MINUTES_MS) {
    return '${differenceMs / MINUTES_MS} minutes';
  }
  if (differenceMs > SECONDS_MS) {
    return '${differenceMs / SECONDS_MS} seconds';
  }
  return '${differenceMs} milliseconds';
}
