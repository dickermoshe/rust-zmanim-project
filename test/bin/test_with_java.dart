import 'dart:async';
import 'dart:io';
import 'dart:math';

import 'package:test_with_java/src/java/kosher_java.g.dart';
import 'package:test_with_java/src/rnd.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:test_with_java/src/rust/api.dart';
import 'package:test_with_java/src/rust/frb_generated.dart';
import 'package:jni/jni.dart';
import 'package:test_with_java/src/test_case.dart';

/// Constants used in testing
const HOURS_MS = 60 * 60 * 1000;
const MINUTES_MS = 60 * 1000;
const SECONDS_MS = 1000;
const MAX_DIFF_MS = 40 * SECONDS_MS;
const MAX_YEAR = 2040;
const MIN_YEAR = 1900;
const TEST_ITERATIONS =
    const int.fromEnvironment('TEST_ITERATIONS', defaultValue: 100);

/// Global random instance
late Random random;

/// Global debug function (if enabled)
late Function(String) debug;

Future<void> main(List<String> args) async {
  // Get the seed from the command line arguments or generate a random one
  int seed;
  if (args.contains('--seed')) {
    seed = int.parse(args[args.indexOf('--seed') + 1]);
    print("Seed: $seed");
  } else {
    seed = DateTime.now().millisecondsSinceEpoch;
  }
  random = Random(seed);
  print("Seed: $seed");

  // Enable the debug function if the --debug flag is present
  if (args.contains('--debug')) {
    debug = (String message) {
      stderr.writeln(message);
    };
  } else {
    debug = (String message) {
      // Do nothing
    };
  }

  // Initialize Rust library
  await RustLib.init(
      externalLibrary: await loadExternalLibrary(ExternalLibraryLoaderConfig(
    stem: 'test_with_java',
    ioDirectory: '../target/release/',
    webPrefix: 'pkg/',
  )));

  // Initialize Java runtime
  Jni.spawn(classPath: ["./java/target/zmanim-2.6.0-SNAPSHOT.jar"]);

  // // Get a list of the timezones that are supported by both Java and Rust
  final javaTimezones =
      ZoneId.getAvailableZoneIds()!.map((e) => e!.toDartString()).toSet();
  final rustTimezones = timezones().toSet();
  final validTimezones = javaTimezones.intersection(rustTimezones).toList();

  final zmanimPresets = presets();
  for (final zman in zmanimPresets) {
    bool allSkipped = true;
    for (var iteration = 0; iteration < TEST_ITERATIONS; iteration++) {
      final testCase = randomTestCase(
        zman: zman,
        iteration: iteration,
        validTimezones: validTimezones,
      );
      final passed = test(testCase);
      if (passed) {
        allSkipped = false;
      }
    }
    if (allSkipped) {
      throw Exception("All tests skipped for ${zman.name()}");
    }
  }
  print("All tests passed");
  exit(0);
}

// Convert a year to a timestamp in seconds
double yearToTimestamp(int year) {
  return (year - 1970) * 365.25 * 24 * 60 * 60;
}

// Generate a random TestCase for a given zman
TestCase randomTestCase(
    {required ZmanimPreset zman,
    required int iteration,
    required List<String> validTimezones}) {
  /// Create a random timestamp in seconds between the minimum and maximum year
  final timestamp =
      random.getDouble(-yearToTimestamp(MIN_YEAR), yearToTimestamp(MAX_YEAR));
  final randomDateTime =
      DateTime.fromMillisecondsSinceEpoch((timestamp * 1000).toInt());
  // TODO: Test with random latitude and longitude
  // final randomLatitude = rnd(-90.0, 90.0);
  // final randomLongitude = rnd(-180.0, 180.0);
  final randomLatitude = random.getDouble(-40.0, 40.0);
  final randomLongitude = random.getDouble(-180.0, 180.0);
  final tz = findTimezone(longitude: randomLongitude, latitude: randomLatitude);

  // if the timezone for this location is not valid, try again
  if (!validTimezones.contains(tz)) {
    return randomTestCase(
        zman: zman, iteration: iteration, validTimezones: validTimezones);
  }
  final randomElevation = random.getDouble(0.0, 1000.0);
  final randomUseElevation = random.nextBool();
  final randomAteretTorahSunsetOffsetMinutes = random.nextInt(60);
  final randomCandleLightingOffsetMinutes = random.nextInt(60);
  final randomUseAstronomicalChatzosForOtherZmanim = random.nextBool();
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

/// Test a given TestCase
/// Throws an exception if the test fails
/// Returns true if the test passes, false if the test was skipped
bool test(TestCase testCase) {
  final int maxDiffMs;
  if (testCase.usesElevation()) {
    // Elevation adds 10 seconds per 100 meters
    maxDiffMs = MAX_DIFF_MS + (testCase.elevation / .1 * SECONDS_MS).toInt();
  } else {
    maxDiffMs = MAX_DIFF_MS;
  }
  final javaZman = calculateJavaZman(testCase);
  final rustZman = calculateRustZman(testCase);

  // Near the poles it is ok if one algorithm is null and the other is not
  if (testCase.nearPoles()) {
    // If either results are null, return
    if (javaZman == null || rustZman == null) {
      return false;
    }
  } else {
    // Otherwise, assert both are null, or neither are null.
    if ((javaZman == null) != (rustZman == null)) {
      throw FailedTest.nullMismatch(testCase, javaZman, rustZman);
    }
  }

  final difference = javaZman!.timestampMs - rustZman!.timestampMs;
  if (difference > maxDiffMs) {
    throw FailedTest.differenceTooLarge(
        testCase, difference, maxDiffMs, javaZman, rustZman);
  }
  return true;
}

class FailedTest implements Exception {
  final TestCase testCase;
  final String message;
  FailedTest(this.testCase, this.message);
  static FailedTest nullMismatch(
      TestCase testCase, ZmanResult? javaZman, ZmanResult? rustZman) {
    final message = [
      "Java: ${javaZman?.toDebugString()}",
      "Rust: ${rustZman?.toDebugString()}",
      "Test Case: ${testCase.toJson()}",
    ].join("\n");
    return FailedTest(testCase, message);
  }

  static FailedTest differenceTooLarge(TestCase testCase, int difference,
      int maxDiffMs, ZmanResult javaZman, ZmanResult rustZman) {
    final message = [
      "Difference too large: $difference ms. Max allowed: $maxDiffMs ms.",
      "Difference: ${formatDifference(difference)}",
      "Java: ${javaZman.toDebugString()}",
      "Rust: ${rustZman.toDebugString()}",
      "Test Case: ${testCase.toJson()}",
    ].join("\n");
    return FailedTest(testCase, message);
  }

  @override
  String toString() => message;
}

class ZmanResult {
  final String formattedDate;
  final int timestampMs;
  ZmanResult(this.formattedDate, this.timestampMs);
  String toDebugString() {
    return "Zman: $formattedDate ($timestampMs)";
  }
}

ZmanResult? calculateJavaZman(TestCase testCase) {
  final javaZoneId = ZoneId.of$1(testCase.timezone.toJString())!;
  final localDate = LocalDate.of$1(testCase.year, testCase.month, testCase.day);
  final location = GeoLocation.new$1("".toJString(), testCase.latitude,
      testCase.longitude, testCase.elevation, javaZoneId);
  final calendar = ComprehensiveZmanimCalendar.new1(location);
  calendar.setUseElevation(testCase.useElevation);
  calendar
      .setCandleLightingOffset(testCase.candleLightingOffsetMinutes.toDouble());
  calendar.setUseAstronomicalChatzosForOtherZmanim(
      testCase.useAstronomicalChatzosForOtherZmanim);
  calendar.setAteretTorahSunsetOffset(
      testCase.ateretTorahSunsetOffsetMinutes.toDouble());
  calendar.setLocalDate(localDate);
  // Invoke the method using the JNI API
  final methodId = calendar.jClass.instanceMethodId(
    testCase.zman.name(),
    r'()Ljava/time/Instant;',
  );
  final result = methodId.call(calendar, $Instant$NullableType$(), []);
  if (result == null) {
    return null;
  }
  final instant = Instant.ofEpochMilli(result.toEpochMilli());
  final ztd = ZonedDateTime.ofInstant(instant, javaZoneId);

  return ZmanResult(ztd!.toString$1()!.toDartString(), instant!.toEpochMilli());
}

ZmanResult? calculateRustZman(TestCase testCase) {
  final result = calculateZman(
    ateretTorahSunsetOffsetMinutes: testCase.ateretTorahSunsetOffsetMinutes,
    candleLightingOffsetMinutes: testCase.candleLightingOffsetMinutes,
    useAstronomicalChatzosForOtherZmanim:
        testCase.useAstronomicalChatzosForOtherZmanim,
    latitude: testCase.latitude,
    longitude: testCase.longitude,
    elevation: testCase.useElevation ? testCase.elevation : 0.0,
    timezone: testCase.timezone,
    randomYear: testCase.year,
    randomMonth: testCase.month,
    randomDay: testCase.day,
    zman: testCase.zman,
  );
  if (result == null) {
    return null;
  }
  final (formattedDate, timestampMs) = result;
  return ZmanResult(formattedDate, timestampMs);
}

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
