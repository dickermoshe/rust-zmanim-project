import 'dart:convert';

import 'package:test_with_java/src/rust/api.dart';

final JsonEncoder _encoder = new JsonEncoder.withIndent('  ');

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
    return _encoder.convert({
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

  bool usesElevation() {
    return useElevation &&
        zman.usesElevation(
            useAstronomicalChatzosForOtherZmanim:
                useAstronomicalChatzosForOtherZmanim);
  }

  bool nearPoles() {
    return latitude.abs() > 75.0;
  }
}
