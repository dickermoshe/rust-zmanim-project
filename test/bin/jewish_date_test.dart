import 'dart:async';
import 'dart:io';
import 'dart:math';

import 'package:test_with_java/src/java/kosher_java.g.dart';
import 'package:test_with_java/src/rnd.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:test_with_java/src/rust/api.dart';
import 'package:test_with_java/src/rust/frb_generated.dart';
import 'package:jni/jni.dart';

import 'package:config/config.dart';

/// Constants used in testing
const HOURS_MS = 60 * 60 * 1000;
const MINUTES_MS = 60 * 1000;
const SECONDS_MS = 1000;

/// Default max difference in milliseconds
const DEFAULT_MAX_DIFF_MS = 30000;

const MAX_GREGORIAN_YEAR = 2040;
const MIN_GREGORIAN_YEAR = 1900;

const MAX_JEWISH_YEAR = MAX_GREGORIAN_YEAR + 3760;
const MIN_JEWISH_YEAR = MIN_GREGORIAN_YEAR + 3760;

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

  for (var iteration = 0; iteration < iterations; iteration++) {
    testGregorianDateToJewishDate();
    testJewishDateToGregorianDate();
    testAddDaysToJewishDate();
    testAddMonthsToJewishDate();
    testAddYearsToJewishDate();
    testMinusDaysToJewishDate();
    testRandomJewishCalendar();
  }
  print("All tests passed");
  exit(0);
}

/// Convert a year to a timestamp in seconds
double yearToTimestamp(int year) {
  return (year - 1970) * 365.25 * 24 * 60 * 60;
}

/// Return a random Jewish date between MIN_YEAR and MAX_YEAR
/// The returned date is not guaranteed to be valid, but it will be between MIN_YEAR and MAX_YEAR
(int, int, int) randomJewishDate() {
  final year =
      random.nextInt(MAX_JEWISH_YEAR - MIN_JEWISH_YEAR + 1) + MIN_JEWISH_YEAR;
  final month = random.nextInt(12) + 1;
  final day = random.nextInt(30) + 1;
  return (year, month, day);
}

/// Return a random Gregorian date between MIN_YEAR and MAX_YEAR
/// The returned date IS guaranteed to be valid
(int, int, int) randomGregorianDate() {
  final timestamp = random.getDouble(
      yearToTimestamp(MIN_GREGORIAN_YEAR), yearToTimestamp(MAX_GREGORIAN_YEAR));
  final randomDateTime =
      DateTime.fromMillisecondsSinceEpoch((timestamp * 1000).toInt());
  return (randomDateTime.year, randomDateTime.month, randomDateTime.day);
}

/// Compare the results of the Rust and Java libraries for a given date
testDates(
    {required (int, int, int) date,
    required String targetDateType,
    (int, int, int)? javaDate,
    (int, int, int)? rustDate}) {
  final (year, month, day) = date;
  switch ((javaDate, rustDate)) {
    case (null, null):
      return;
    case (null, (int _, int _, int _)):
      throw Exception("Converting ${(
        year,
        month,
        day
      )} to $targetDateType date failed. Rust result: $rustDate, Java result: $javaDate");
    case ((int _, int _, int _), null):
      throw Exception("Converting ${(
        year,
        month,
        day
      )} to $targetDateType date failed. Rust result: $rustDate, Java result: $javaDate");
    case ((int _, int _, int _), (int _, int _, int _)):
      if (rustDate != javaDate) {
        throw Exception("Converting ${(
          year,
          month,
          day
        )} to $targetDateType date failed. Rust result: $rustDate, Java result: $javaDate");
      }
      return;
  }
}

void testGregorianDateToJewishDate() {
  final (year, month, day) = randomGregorianDate();
  final rustResult =
      gregorianDateToJewishDate(year: year, month: month, day: day);
  final javaResult = javaGregorianDateToJewishDate(year, month, day);
  testDates(
      date: (year, month, day),
      targetDateType: "Jewish",
      javaDate: javaResult,
      rustDate: rustResult);
}

void testJewishDateToGregorianDate() {
  final (year, month, day) = randomJewishDate();
  final rustResult =
      jewishDateToGregorianDate(year: year, month: month, day: day);
  final javaResult = javaJewishDateToGregorianDate(year, month, day);
  testDates(
      date: (year, month, day),
      targetDateType: "Gregorian",
      javaDate: javaResult,
      rustDate: rustResult);
}

void testAddDaysToJewishDate() {
  final (year, month, day) = randomJewishDate();
  final daysToAdd = random.nextInt(600) + 1;
  final rustResult = addDaysToJewishDate(
      year: year, month: month, day: day, daysToAdd: daysToAdd);
  final javaResult = javaAddDaysToJewishDate(year, month, day, daysToAdd);
  testDates(
      date: (year, month, day),
      targetDateType: "Jewish after adding $daysToAdd days",
      javaDate: javaResult,
      rustDate: rustResult);
}

void testMinusDaysToJewishDate() {
  final (year, month, day) = randomJewishDate();
  final daysToAdd = random.nextInt(600) + 1;

  final rustResult = addDaysToJewishDate(
    year: year,
    month: month,
    day: day,
    daysToAdd: -daysToAdd, // We subtract in rust by adding a negative number
  );
  final javaResult = javaMinusDaysToJewishDate(year, month, day, daysToAdd);
  testDates(
      date: (year, month, day),
      targetDateType: "Jewish after subtracting $daysToAdd days",
      javaDate: javaResult,
      rustDate: rustResult);
}

void testAddMonthsToJewishDate() {
  final (year, month, day) = randomJewishDate();
  final monthsToAdd = random.nextInt(120) + 1;
  final rustResult = addMonthsToJewishDate(
      year: year, month: month, day: day, monthsToAdd: monthsToAdd);
  final javaResult = javaAddMonthsToJewishDate(year, month, day, monthsToAdd);
  testDates(
      date: (year, month, day),
      targetDateType: "Jewish after adding $monthsToAdd months",
      javaDate: javaResult,
      rustDate: rustResult);
}

void testAddYearsToJewishDate() {
  final (year, month, day) = randomJewishDate();
  final yearsToAdd = random.nextInt(60) + 1;
  final rustResult = addYearsToJewishDate(
      year: year, month: month, day: day, yearsToAdd: yearsToAdd);
  final javaResult = javaAddYearsToJewishDate(year, month, day, yearsToAdd);
  testDates(
      date: (year, month, day),
      targetDateType: "Jewish after adding $yearsToAdd years",
      javaDate: javaResult,
      rustDate: rustResult);
}

(int, int, int)? javaGregorianDateToJewishDate(int year, int month, int day) {
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
    } catch (_) {}
  }
  return null;
}

(int, int, int)? javaAddDaysToJewishDate(
    int year, int month, int day, int daysToAdd) {
  for (final _ in Iterable.generate(3)) {
    try {
      final jewishDate = JewishDate.new$1(year, month, day);
      jewishDate.addDays(daysToAdd);
      return (
        jewishDate.getJewishYear(),
        jewishDate.getJewishMonth(),
        jewishDate.getJewishDayOfMonth()
      );
    } catch (_) {}
  }
  return null;
}

(int, int, int)? javaMinusDaysToJewishDate(
    int year, int month, int day, int daysToSubtract) {
  for (final _ in Iterable.generate(3)) {
    try {
      final jewishDate = JewishDate.new$1(year, month, day);
      jewishDate.minusDays(daysToSubtract);
      return (
        jewishDate.getJewishYear(),
        jewishDate.getJewishMonth(),
        jewishDate.getJewishDayOfMonth()
      );
    } catch (_) {}
  }
  return null;
}

(int, int, int)? javaAddMonthsToJewishDate(
    int year, int month, int day, int monthsToAdd) {
  for (final _ in Iterable.generate(3)) {
    try {
      final jewishDate = JewishDate.new$1(year, month, day);
      jewishDate.addMonths(monthsToAdd);
      return (
        jewishDate.getJewishYear(),
        jewishDate.getJewishMonth(),
        jewishDate.getJewishDayOfMonth()
      );
    } catch (_) {}
  }
  return null;
}

(int, int, int)? javaAddYearsToJewishDate(
    int year, int month, int day, int yearsToAdd) {
  for (final _ in Iterable.generate(3)) {
    try {
      final jewishDate = JewishDate.new$1(year, month, day);
      // Skips to Adar II in a leap year
      jewishDate.addYears(yearsToAdd, false);
      return (
        jewishDate.getJewishYear(),
        jewishDate.getJewishMonth(),
        jewishDate.getJewishDayOfMonth()
      );
    } catch (_) {}
  }
  return null;
}

(int, int, int)? javaJewishDateToGregorianDate(int year, int month, int day) {
  // Try up to 3 times to get the result
  for (final _ in Iterable.generate(3)) {
    try {
      final jewishDate = JewishDate.new$1(year, month, day);
      final localDate = jewishDate.getLocalDate();
      if (localDate == null) {
        continue;
      }
      return (
        localDate.getYear(),
        localDate.getMonthValue(),
        localDate.getDayOfMonth()
      );
    } catch (_) {}
  }
  return null;
}

void testRandomJewishCalendar() {
  final (year, month, day) = randomJewishDate();
  final inIsrael = random.nextBool();
  final useModernHolidays = random.nextBool();
  final calendar = JewishCalendar.new$5(year, month, day, inIsrael);
  calendar.setUseModernHolidays(useModernHolidays);

  int? dayOfChanukah = calendar.getDayOfChanukah();
  if (dayOfChanukah == -1) {
    dayOfChanukah = null;
  }
  int? dayOfOmer = calendar.getDayOfOmer();
  if (dayOfOmer == -1) {
    dayOfOmer = null;
  }

  final getUpcomingParshah = getParshaIndex(calendar.getUpcomingParshah())!;
  final getSpecialShabbos = getParshaIndex(calendar.getSpecialShabbos());
  final getParshah = getParshaIndex(calendar.getParshah());

  final testResults = JavaJewishCalendarTestResults(
    getParshah: getParshah,
    getUpcomingParshah: getUpcomingParshah,
    getSpecialShabbos: getSpecialShabbos,
    isBirkasHachamah: calendar.isBirkasHachamah(),
    getYomTovIndex: calendar.getYomTovIndex(),
    isAssurBemelacha: calendar.isAssurBemelacha(),
    hasCandleLighting: calendar.hasCandleLighting(),
    isAseresYemeiTeshuva: calendar.isAseresYemeiTeshuva(),
    isYomKippurKatan: calendar.isYomKippurKatan(),
    isBeHaB: calendar.isBeHaB(),
    isTaanisBechoros: calendar.isTaanisBechoros(),
    getDayOfChanukah: dayOfChanukah,
    isRoshChodesh: calendar.isRoshChodesh(),
    isMacharChodesh: calendar.isMacharChodesh(),
    isShabbosMevorchim: calendar.isShabbosMevorchim(),
    getDayOfOmer: dayOfOmer,
  );
  testJewishCalendar(
      year: year,
      month: month,
      day: day,
      inIsrael: inIsrael,
      useModernHolidays: useModernHolidays,
      java: testResults);
}

/// A helper function to get the index of a parsha enum
int? getParshaIndex(JewishCalendar$Parsha? parsha) {
  if (parsha == null) {
    return null;
  }
  final parshaClass = JClass.forName(
      r'com/kosherjava/zmanim/hebrewcalendar/JewishCalendar$Parsha');
  final parshaOrdinal = parshaClass
      .instanceMethodId('ordinal', '()I')
      .call(parsha, jint.type, []);
  if (parshaOrdinal == 0) {
    return null;
  }
  return parshaOrdinal - 1;
}
