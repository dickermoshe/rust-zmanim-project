import 'package:kosher_rust_for_dart/kosher_rust_for_dart.dart';
import 'package:test/test.dart';

void main() {
  group('kosher_rust_for_dart smoke', () {
    test('gregorian/hebrew round trip via Calendar', () {
      final calendar = Calendar();
      final civil = CivilDate(year: 2024, month: 1, day: 20);

      final hebrew = calendar.gregorianToHebrew(civil);
      expect(hebrew, isNotNull);
      expect(hebrew!.year, 5784);
      expect(hebrew.month, 5);
      expect(hebrew.day, 10);

      final back = calendar.hebrewToGregorian(hebrew);
      expect(back, isNotNull);
      expect(back!.year, civil.year);
      expect(back.month, civil.month);
      expect(back.day, civil.day);
    });

    test('daf yomi bavli via Limudim', () {
      final limudim = Limudim();
      final daf = limudim.dafYomiBavli(2017, 12, 28);

      expect(daf, isNotNull);
      expect(daf!.tractate, TractateCode.shevuos);
      expect(daf.page, 30);
    });

    test('preset count via ZmanPresets', () {
      final presets = ZmanPresets();
      expect(presets.presetCount(), 167);
    });
  });
}
