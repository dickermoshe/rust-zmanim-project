/// A collection of helpful extensions for the dart:math Random class.
import 'dart:math';

/// A collection of helpful extensions for the dart:math Random class.
extension RndExtensions on Random {
  /// Returns a random `double` between min and max.
  double getDouble(double min, double max) {
    return min + nextDouble() * (max - min);
  }

  /// Returns a random item from the specified `List`.
  /// If `remove` is true, the item is removed from the list.
  T getItem<T>(List<T> list, {bool remove = false}) {
    final int i = nextInt(list.length);
    return remove ? list.removeAt(i) : list[i];
  }
}
