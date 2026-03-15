import 'dart:math';

int _seed = DateTime.now().millisecondsSinceEpoch;

/// Globally accessible instance of `Random`.
/// Makes using random values as easy as `rnd(20)` or `rnd.getBool()`.
Random rnd = Random(_seed);

/// Sets the seed of the `rnd` global `Random` instance.
set rndSeed(int seed) => rnd = Random(_seed = seed);

/// Gets the seed of the `rnd` global `Random` instance.
int get rndSeed => _seed;

/// A collection of helpful extensions for the dart:math Random class.
extension RndExtensions on Random {
  /// Allows you to call a Random instance directly to get a random `double` between min and max.
  /// If only one param is passed, a value between 0 and it is returned. Ex. `rnd(10)` returns 0-10.
  /// If no params are passed, a value between 0 and 1 is returned. Ex. `rnd()` returns 0-1.
  double call([double? min, double? max]) {
    if (max == null) {
      max = min ?? 1.0;
      min = 0.0;
    }
    return getDouble(min!, max);
  }

  /// Returns a random `int` between min and max.
  int getInt(int min, int max) {
    final span = max - min + 1;
    final high = nextInt(1 << 32);
    final low = nextInt(1 << 32);
    final rand = ((high << 32) | low) & 0x7fffffffffffffff;
    return min + rand % span;
  }

  /// Returns a random `double` between min and max.
  double getDouble(double min, double max) {
    return min + nextDouble() * (max - min);
  }

  /// Returns a `bool`, where `chance` specifies the chance of returning `true`.
  /// For example, `getBool(0.8)`, would have an 80% chance to return `true`.
  bool getBool([double chance = 0.5]) {
    return nextDouble() < chance;
  }

  /// Returns `0` or `1`, where `chance` specifies the chance of it returning `1`.
  /// For example, `getBit(0.8)`, would have an 80% chance to return `1`.
  int getBit([double chance = 0.5]) {
    return nextDouble() < chance ? 1 : 0;
  }

  /// Returns `-1` or `1`, where `chance` specifies the chance of it returning `1`.
  /// For example, `getSign(0.8)`, would have an 80% chance to return `1`.
  int getSign([double chance = 0.5]) {
    return nextDouble() < chance ? 1 : -1;
  }

  /// Returns a random `double` between `0` and `360`.
  double getDeg() {
    return nextDouble() * 360.0;
  }

  /// Returns a random `double` between `0` and `pi * 2`.
  double getRad() {
    return nextDouble() * pi * 2.0;
  }

  /// Returns a random item from the specified `List`.
  /// If `remove` is true, the item is removed from the list.
  T getItem<T>(List<T> list, {bool remove = false}) {
    final int i = nextInt(list.length);
    return remove ? list.removeAt(i) : list[i];
  }

  /// Randomizes the order of the specified `List`.
  /// If `copy` is true, returns a shuffled copy of the list, if false, it shuffles and returns the original.
  List<T> shuffle<T>(List<T> list, {bool copy = false}) {
    if (copy) {
      list = [...list];
    }
    for (int i = 0, l = list.length; i < l; i++) {
      int j = nextInt(l);
      if (j == i) {
        continue;
      }
      T item = list[j];
      list[j] = list[i];
      list[i] = item;
    }
    return list;
  }
}

/// Hue values for primary and secondary colors. For use with `getColor()`.
class Hue {
  static const double red = 0.0;
  static const double green = 120.0;
  static const double blue = 240.0;

  static const double yellow = 60.0;
  static const double cyan = 180.0;
  static const double magenta = 300.0;
}
