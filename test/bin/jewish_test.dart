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

/// Default max difference in milliseconds
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

  // Initialize Rust library
  await RustLib.init(
      externalLibrary: await loadExternalLibrary(ExternalLibraryLoaderConfig(
    stem: 'test_with_java',
    ioDirectory: '../target/release/',
    webPrefix: 'pkg/',
  )));

  // Initialize Java runtime
  Jni.spawn(classPath: ["./java/target/zmanim-2.6.0-SNAPSHOT.jar"]);
}

/// Convert a year to a timestamp in seconds
double yearToTimestamp(int year) {
  return (year - 1970) * 365.25 * 24 * 60 * 60;
}

/// Return a random Jewish date between MIN_YEAR and MAX_YEAR
/// The returned date is not guaranteed to be valid, but it will be between MIN_YEAR and MAX_YEAR
(int, int, int) randomJewishDate() {
  final year = random.nextInt(MAX_YEAR - MIN_YEAR + 1) + MIN_YEAR;
  final month = random.nextInt(12) + 1;
  final day = random.nextInt(30);
  return (year, month, day);
}

/// Return a random Gregorian date between MIN_YEAR and MAX_YEAR
/// The returned date IS guaranteed to be valid
(int, int, int) randomGregorianDate() {
  final timestamp =
      random.getDouble(yearToTimestamp(MIN_YEAR), yearToTimestamp(MAX_YEAR));
  final randomDateTime =
      DateTime.fromMillisecondsSinceEpoch((timestamp * 1000).toInt());
  return (randomDateTime.year, randomDateTime.month, randomDateTime.day);
}

void testJewishDateToGregorianDate() {
  final (year, month, day) = randomJewishDate();
  final rustResult =
      gregorianDateToJewishDate(year: year, month: month, day: day);
  final javaResult = javaJewishDateToGregorianDate(year, month, day);
  switch ((rustResult, javaResult)) {
    case (null, null):
      return;
    case (null, (int _, int _, int _)):
      throw Exception("Converting ${(
        year,
        month,
        day
      )} to Gregorian date failed. Rust result: $rustResult, Java result: $javaResult");
    case ((int _, int _, int _), null):
      throw Exception("Converting ${(
        year,
        month,
        day
      )} to Gregorian date failed. Rust result: $rustResult, Java result: $javaResult");
    case ((int _, int _, int _), (int _, int _, int _)):
      if (rustResult != javaResult) {
        throw Exception("Converting ${(
          year,
          month,
          day
        )} to Gregorian date failed. Rust result: $rustResult, Java result: $javaResult");
      }
      return;
  }
}

(int, int, int)? javaJewishDateToGregorianDate(int year, int month, int day) {
  // Try up to 3 times to get the result
  for (final _ in Iterable.generate(3)) {
    try {
      final localDate = LocalDate.of$1(year, month, day);
      final gregorianDate = JewishDate.new$4(localDate);
      return (
        gregorianDate.getJewishYear(),
        gregorianDate.getJewishMonth(),
        gregorianDate.getJewishDayOfMonth()
      );
    } catch (e) {
      print("Error: $e");
    }
  }
  return null;
}
