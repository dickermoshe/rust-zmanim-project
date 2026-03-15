import 'dart:async';
import 'dart:math';

import 'package:frb_example_dart_minimal/rnd.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:frb_example_dart_minimal/kosher_java.g.dart';
import 'package:frb_example_dart_minimal/src/rust/api.dart';
import 'package:frb_example_dart_minimal/src/rust/frb_generated.dart';
import 'package:jni/_internal.dart';
import 'package:test/test.dart';
import 'package:jni/jni.dart';
import 'package:jni/_internal.dart' as jni$_;
import 'package:jni/jni.dart' as jni$_;

const YEARS = 200;
const YEARS_MS = YEARS * 365 * 24 * 60 * 60 * 1000;
// ONYL TEST TIMEZONES AND IN LAT LONGS THAT MAKE SENSE!!!!
Future<void> main() async {
  print('Rnd seed: ${rndSeed}');

  await RustLib.init(
      externalLibrary: await loadExternalLibrary(ExternalLibraryLoaderConfig(
    stem: 'frb_example_dart_minimal',
    ioDirectory: '../target/release/',
    webPrefix: 'pkg/',
  )));
  Jni.spawn(classPath: [
    "./kosher-java/target/zmanim-2.6.0-SNAPSHOT.jar",
  ]);
  group('calculate', () {
    final java_timezones =
        ZoneId.getAvailableZoneIds()!.map((e) => e!.toDartString()).toSet();
    final rust_timezones = timezones().toSet();
    final valid_timezones =
        java_timezones.intersection(rust_timezones).toList();
    final zmanimPresets = presets();
    for (final zman in zmanimPresets) {
      final name = zman.name();
      test(name, () async {
        for (var i = 0; i < 100; i++) {
          // final randomZmanimPreset = rnd.getItem(zmanimPresets);
          final randomZmanimPreset = zmanimPresets[0];
          final randomMillisecondsSinceEpoch = rnd.getInt(YEARS_MS, -YEARS_MS);
          // TODO: Test with random latitude and longitude
          // final randomLatitude = rnd(-90.0, 90.0);
          // final randomLongitude = rnd(-180.0, 180.0);
          final randomLatitude = rnd(-40.0, 40.0);
          final randomLongitude = rnd(-180.0, 180.0);
          final tz = await findTimezone(
              longitude: randomLongitude, latitude: randomLatitude);
          if (!valid_timezones.contains(tz)) {
            continue;
          }
          // TODO: Test with random elevation
          // final randomElevation = rnd(0.0, 400.0);
          final randomElevation = 0.0;

          final randomAteretTorahSunsetOffsetMinutes = rnd.getInt(0, 60);
          final randomCandleLightingOffsetMinutes = rnd.getInt(0, 60);
          final randomUseAstronomicalChatzosForOtherZmanim = rnd.getBool();
          final javaZman = await calculateJavaZman(
            ateretTorahSunsetOffsetMinutes:
                randomAteretTorahSunsetOffsetMinutes,
            candleLightingOffsetMinutes: randomCandleLightingOffsetMinutes,
            useAstronomicalChatzosForOtherZmanim:
                randomUseAstronomicalChatzosForOtherZmanim,
            latitude: randomLatitude,
            longitude: randomLongitude,
            elevation: randomElevation,
            timezone: tz,
            millisecondsSinceEpoch: randomMillisecondsSinceEpoch,
            zman: randomZmanimPreset,
          );
          final rustZman = calculateZman(
            ateretTorahSunsetOffsetMinutes:
                randomAteretTorahSunsetOffsetMinutes,
            candleLightingOffsetMinutes: randomCandleLightingOffsetMinutes,
            useAstronomicalChatzosForOtherZmanim:
                randomUseAstronomicalChatzosForOtherZmanim,
            latitude: randomLatitude,
            longitude: randomLongitude,
            elevation: randomElevation,
            timezone: tz,
            millisecondsSinceEpoch: randomMillisecondsSinceEpoch,
            zman: randomZmanimPreset,
          );
          if (javaZman == null || rustZman == null) {
            continue;
          }

          final (formattedJavaTimestamp, javaTimestampMs) = javaZman;
          final (formattedRustTimestamp, rustTimestampMs) = rustZman;

          expect(
            javaTimestampMs,
            closeTo(rustTimestampMs, 24 * 60 * 60 * 1000), // Less than 1 day
            formatter: (actual, matcher, reason, matchState, verbose) {
              final timestamp = timestampAtTimezone(
                  timezone: tz,
                  millisecondsSinceEpoch: randomMillisecondsSinceEpoch);
              final reason = StringBuffer();
              reason.writeln('Testing ${name} at iteration $i');
              reason.writeln('Timezone: $tz');
              reason.writeln('Latitude: $randomLatitude');
              reason.writeln('Longitude: $randomLongitude');
              reason.writeln('Elevation: $randomElevation');
              reason.writeln('Date: $timestamp');
              reason.writeln('Timestamp: $randomMillisecondsSinceEpoch');
              reason.writeln('Java Result Ms: $javaTimestampMs');
              reason.writeln('Java Result Date: $formattedJavaTimestamp');
              reason.writeln('Rust Result Ms: $rustTimestampMs');
              reason.writeln('Rust Result Date: $formattedRustTimestamp');
              reason
                  .writeln('Difference: ${javaTimestampMs - rustTimestampMs}');
              return reason.toString();
            },
          );
        }
      });
    }
  });
}

Future<(String, int)?> calculateJavaZman(
    {required int ateretTorahSunsetOffsetMinutes,
    required int candleLightingOffsetMinutes,
    required bool useAstronomicalChatzosForOtherZmanim,
    required double latitude,
    required double longitude,
    required double elevation,
    required String timezone,
    required int millisecondsSinceEpoch,
    required ZmanimPreset zman}) async {
  final javaZoneId = ZoneId.of$1(timezone.toJString())!;
  final instant = Instant.ofEpochMilli(millisecondsSinceEpoch)!;
  final zonedDateTime = ZonedDateTime.ofInstant(instant, javaZoneId)!;
  final location = GeoLocation.new$1(
      "".toJString(), latitude, longitude, elevation, javaZoneId);
  final calendar = ComprehensiveZmanimCalendar.new1(location);
  calendar.setZonedDateTime(zonedDateTime);
  calendar.getAlos120();
  // Invoke the method using the JNI API
  final methodId = calendar.jClass.instanceMethodId(
    zman.name(),
    r'()Ljava/time/Instant;',
  ) as JMethodIDPtr;
  final fn = jni$_.ProtectedJniExtensions.lookup<
          jni$_.NativeFunction<
              jni$_.JniResult Function(
                jni$_.Pointer<jni$_.Void>,
                jni$_.JMethodIDPtr,
              )>>('globalEnv_CallObjectMethod')
      .asFunction<
          jni$_.JniResult Function(
            jni$_.Pointer<jni$_.Void>,
            jni$_.JMethodIDPtr,
          )>();
  final result = fn(calendar.reference.pointer, methodId)
      .object<Instant?>(const $Instant$NullableType$());
  if (result?.reference.isNull ?? false) {
    return null;
  }
  if (result == null) {
    return null;
  }
  final timestamp = result.atZone(javaZoneId)!.toString$1()!.toDartString();
  return (timestamp, result.toEpochMilli());
}
