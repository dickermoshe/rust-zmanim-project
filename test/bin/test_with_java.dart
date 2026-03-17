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

import 'package:config/config.dart';

/// Constants used in testing
const HOURS_MS = 60 * 60 * 1000;
const MINUTES_MS = 60 * 1000;
const SECONDS_MS = 1000;
const DEFAULT_MAX_DIFF_MS = 30000;
const MAX_YEAR = 2040;
const MIN_YEAR = 1900;

/// Global random instance
late Random random;

/// Default seed is the current time in milliseconds since epoch
int defaultSeed() => DateTime.now().millisecondsSinceEpoch;

/// Options are defineable as enums as well as regular lists.
///
/// The enum approach is more distinct and type safe.
/// The list approach is more dynamic and permits non-const initialization.
enum TestOption<V> implements OptionDefinition<V> {
  seed(IntOption(
    argName: 'seed',
    argAbbrev: 's',
    helpText: 'The seed to use for the random number generator',
    min: 0,
    fromDefault: defaultSeed,
  )),
  methodFilter(StringOption(
    argName: 'filter',
    argAbbrev: 'f',
    helpText: 'Filter the methods to test',
  )),
  iterations(IntOption(
    argName: 'iterations',
    argAbbrev: 'i',
    helpText: 'The number of iterations to test',
    min: 1,
    max: 100000,
    defaultsTo: 1000,
    envName: 'TEST_ITERATIONS',
  ));

  const TestOption(this.option);

  @override
  final ConfigOptionBase<V> option;
}

Future<void> main(List<String> args) async {
  final configuration = Configuration.resolve(
      options: TestOption.values, args: args, env: Platform.environment);
  final seed = configuration.value(TestOption.seed);
  random = Random(seed);
  print("Seed: $seed");

  final iterations = configuration.value(TestOption.iterations);
  print("Iterations: $iterations");

  final methodFilter = configuration.optionalValue(TestOption.methodFilter);
  print("Method filter: $methodFilter");

  // Initialize Rust library
  await RustLib.init(
      externalLibrary: await loadExternalLibrary(ExternalLibraryLoaderConfig(
    stem: 'test_with_java',
    ioDirectory: '../target/release/',
    webPrefix: 'pkg/',
  )));

  // Initialize Java runtime
  Jni.spawn(classPath: ["./java/target/zmanim-2.6.0-SNAPSHOT.jar"]);

  // Get a list of the timezones that are supported by both Java and Rust
  final javaTimezones =
      ZoneId.getAvailableZoneIds()!.map((e) => e!.toDartString()).toSet();
  final rustTimezones = timezones().toSet();
  final validTimezones = javaTimezones.intersection(rustTimezones).toList();
  final zmanimPresets = presets()
      .where((e) => methodFilter == null || e.name().contains(methodFilter))
      .toList();

  for (var iteration = 0; iteration < iterations; iteration++) {
    for (final zman in zmanimPresets) {
      final testCase = randomTestCase(
        zman: zman,
        iteration: iteration,
        validTimezones: validTimezones,
      );
      test(testCase);
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

  // Going any higher will make the difference between NOAA and SPA start to become
  // too high to do any meaningful testing.
  final randomLatitude = random.getDouble(-60.0, 60.0);
  final randomLongitude = random.getDouble(-180.0, 180.0);
  final tz = findTimezone(longitude: randomLongitude, latitude: randomLatitude);

  // if the timezone for this location is not valid, try again
  if (!validTimezones.contains(tz)) {
    return randomTestCase(
        zman: zman, iteration: iteration, validTimezones: validTimezones);
  }
  final randomElevation = random.getDouble(0.0, 4000.0);
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
  final javaZman = calculateJavaZman(testCase);
  final rustZman = calculateRustZman(testCase);

  // Zmanim related to Chametz are only returned by Java if it is Erev Pesach,
  // However, Rust will return the zmanim for any date.
  final isChametzZman = testCase.zmanName.contains("Chametz");

  // Near the poles, tiny algorithm differences can make one implementation return null while the other returns a concrete time.
  final nearPoles = testCase.latitude.abs() > 50.0;

  switch ((javaZman, rustZman)) {
    case (null, null):
      return false;
    case (null, ZmanResult()) || (ZmanResult(), null):
      if (isChametzZman || nearPoles) {
        return false;
      }
      throw FailedTest.nullMismatch(testCase, javaZman, rustZman);
    case (ZmanResult javaZman, ZmanResult rustZman):
      final difference = (javaZman.timestampMs - rustZman.timestampMs).abs();
      if (difference > DEFAULT_MAX_DIFF_MS) {
        throw FailedTest.differenceTooLarge(
            testCase, difference, DEFAULT_MAX_DIFF_MS, javaZman, rustZman);
      }
      return true;
  }
  throw StateError('Unreachable');
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
  /// There is an off error with JNI where we will get a JniException
  /// for seemingly no reason. We should try up to 3 times to get the result.
  for (final i in Iterable.generate(3)) {
    try {
      final javaZoneId = ZoneId.of$1(testCase.timezone.toJString())!;
      final localDate =
          LocalDate.of$1(testCase.year, testCase.month, testCase.day);
      final location = GeoLocation.new$1("".toJString(), testCase.latitude,
          testCase.longitude, testCase.elevation, javaZoneId);
      final calendar = ComprehensiveZmanimCalendar.new1(location);
      calendar.setUseElevation(testCase.useElevation);
      calendar.setCandleLightingOffset(
          testCase.candleLightingOffsetMinutes.toDouble());

      // We compare `CHATZOS_ASTRONOMICAL` with `getChatzos`.
      // They will only be functionally equivalent if `useAstronomicalChatzos` is set to true.
      calendar.setUseAstronomicalChatzos(true);
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
      final milliseconds = result.toEpochMilli();
      final instant = Instant.ofEpochMilli(milliseconds);
      final ztd = ZonedDateTime.ofInstant(instant, javaZoneId);

      return ZmanResult(ztd!.toString$1()!.toDartString(), milliseconds);
    } on JniException catch (_) {
      if (i == 2) {
        rethrow;
      }
    }
  }

  return null;
}

/// Calculate the zman using the Rust library
ZmanResult? calculateRustZman(TestCase testCase) {
  final result = calculateZman(
    useElevation: testCase.useElevation,
    ateretTorahSunsetOffsetMinutes: testCase.ateretTorahSunsetOffsetMinutes,
    candleLightingOffsetMinutes: testCase.candleLightingOffsetMinutes,
    useAstronomicalChatzosForOtherZmanim:
        testCase.useAstronomicalChatzosForOtherZmanim,
    latitude: testCase.latitude,
    longitude: testCase.longitude,
    elevation: testCase.elevation,
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
