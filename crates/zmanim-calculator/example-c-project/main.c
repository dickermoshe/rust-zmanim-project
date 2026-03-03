#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "../bindings/c/zmanim_calendar.h"

static int fail(const char *message) {
    fprintf(stderr, "FAIL: %s\n", message);
    return 1;
}

int main(void) {
    optioni32 timezone = {.t = 2 * 60 * 60, .is_some = 1};

    optionlocation valid_location = new_location(31.778, 35.235, 800.0, timezone);
    if (!valid_location.is_some) {
        return fail("new_location(valid) returned none");
    }

    optionlocation invalid_location = new_location(200.0, 35.235, 800.0, timezone);
    if (invalid_location.is_some) {
        return fail("new_location(invalid latitude) unexpectedly returned some");
    }

    calculatorconfig config = {
        .candle_lighting_offset_seconds = 18 * 60,
        .use_astronomical_chatzos_for_other_zmanim = false,
        .ateret_torah_sunset_offset_seconds = 40 * 60,
    };

    optionzmanimcalculator calculator =
        new_calculator(valid_location.t, 2026, 3, 1, config);
    if (!calculator.is_some) {
        return fail("new_calculator(valid) returned none");
    }

    optioni64 sunrise_value = sunrise(&calculator.t);
    optioni64 sunset_value = sunset(&calculator.t);
    optioni64 alos_72_value = alos_72_minutes(&calculator.t);

    if (!sunrise_value.is_some) {
        return fail("sunrise returned none");
    }
    if (!sunset_value.is_some) {
        return fail("sunset returned none");
    }
    if (!alos_72_value.is_some) {
        return fail("alos_72_minutes returned none");
    }

    if (sunrise_value.t >= sunset_value.t) {
        return fail("sunrise should be before sunset");
    }
    if (alos_72_value.t >= sunrise_value.t) {
        return fail("alos_72_minutes should be before sunrise");
    }

    printf("PASS\n");
    printf("sunrise: %" PRId64 "\n", sunrise_value.t);
    printf("sunset : %" PRId64 "\n", sunset_value.t);
    printf("alos72 : %" PRId64 "\n", alos_72_value.t);
    return 0;
}
