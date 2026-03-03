// Test file for limudim-wasm - Daf Yomi Yerushalmi only
// Run with: bun test

import { daf_yomi_yerushalmi } from "./pkg/limudim_wasm.js";
import { HDate } from "@hebcal/core";
import { yerushalmiYomi, vilna } from "@hebcal/learning";
import { faker } from "@faker-js/faker";
import { expect, test, describe } from "bun:test";

// Mapping from our tractate names to hebcal's tractate names (Yerushalmi)
const TRACTATE_MAP: Record<string, string> = {
    // Yerushalmi-specific spellings
    Berachos: "Berakhot",
    Berakhot: "Berakhot",
    Peah: "Peah",
    Demai: "Demai",
    Kilayim: "Kilayim",
    Sheviis: "Sheviit",
    Sheviit: "Sheviit",
    Terumos: "Terumot",
    Terumot: "Terumot",
    Maasros: "Maasrot",
    Maasrot: "Maasrot",
    MaaserSheni: "Maaser Sheni",
    "Maaser Sheni": "Maaser Sheni",
    Chalah: "Challah",
    Challah: "Challah",
    Orlah: "Orlah",
    Bikurim: "Bikkurim",
    Bikkurim: "Bikkurim",
    Shabbos: "Shabbat",
    Shabbat: "Shabbat",
    Eruvin: "Eruvin",
    Pesachim: "Pesachim",
    Beitzah: "Beitzah",
    RoshHashanah: "Rosh Hashanah",
    "Rosh Hashanah": "Rosh Hashanah",
    Yoma: "Yoma",
    Sukkah: "Sukkah",
    Taanis: "Taanit",
    Taanit: "Taanit",
    Shekalim: "Shekalim",
    Megillah: "Megillah",
    Chagigah: "Chagigah",
    MoedKatan: "Moed Katan",
    "Moed Katan": "Moed Katan",
    Yevamos: "Yevamot",
    Yevamot: "Yevamot",
    Kesubos: "Ketubot",
    Ketubot: "Ketubot",
    Sotah: "Sotah",
    Nedarim: "Nedarim",
    Nazir: "Nazir",
    Gitin: "Gittin",
    Gittin: "Gittin",
    Kiddushin: "Kiddushin",
    BavaKamma: "Bava Kamma",
    "Bava Kamma": "Bava Kamma",
    BavaMetzia: "Bava Metzia",
    "Bava Metzia": "Bava Metzia",
    BavaBasra: "Bava Batra",
    "Bava Batra": "Bava Batra",
    Sanhedrin: "Sanhedrin",
    Makkos: "Makkot",
    Makkot: "Makkot",
    Shevuos: "Shevuot",
    Shevuot: "Shevuot",
    AvodahZarah: "Avodah Zarah",
    "Avodah Zarah": "Avodah Zarah",
    Horiyos: "Horayot",
    Horayot: "Horayot",
    Niddah: "Niddah",
};

function mapTractate(ourTractate: string): string {
    return TRACTATE_MAP[ourTractate] || ourTractate;
}

interface DafResult {
    tractate: string;
    page: number;
}

// Helper to generate random dates within a range
function randomDate(from: string, to: string): Date {
    return faker.date.between({ from, to });
}

describe("Daf Yomi Yerushalmi", () => {


    
    test("matches @hebcal/learning for random dates 1980-2060", () => {
        // Skip day postponement logic doesn't match hebcal yet
        // When Tisha B'Av falls on Shabbat, hebcal's handling differs from ours
        for (let i = 0; i < 200; i++) {
            const date = randomDate("1900-01-01", "2060-12-31");
            
            // Extract year, month, day to avoid timezone issues
            const y = date.getUTCFullYear();
            const m = date.getUTCMonth() + 1;
            const d = date.getUTCDate();
            
            // Create a clean UTC date for hebcal
            const utcDate = new Date(Date.UTC(y, m - 1, d));
            const hd = new HDate(utcDate);

            let wasmResult: DafResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = daf_yomi_yerushalmi(y, m, d);
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = yerushalmiYomi(new HDate(utcDate), vilna);
            } catch (e) {
                hebcalResult = null;
            }

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalResult === null) continue;
            
            if (!wasmResult || !hebcalResult) {
                const dateStr = `${y}-${m.toString().padStart(2,'0')}-${d.toString().padStart(2,'0')}`;
                const hebrewMonth = hd.getMonth();
                const hebrewDay = hd.getDate();
                console.log(`\nNull mismatch on ${dateStr} (Hebrew: ${hebrewMonth}/${hebrewDay}):`);
                console.log(`  WASM: ${wasmResult ? `${wasmResult.tractate}:${wasmResult.page}` : 'null'}`);
                console.log(`  Hebcal: ${hebcalResult ? `${hebcalResult.name}:${hebcalResult.blatt}` : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalResult).not.toBeNull();

            const wasmMapped = `${mapTractate(wasmResult!.tractate)}:${wasmResult!.page}`;
            const hebcalMapped = `${mapTractate(hebcalResult.name)}:${hebcalResult.blatt}`;

            if (wasmMapped !== hebcalMapped) {
                const dateStr = `${y}-${m.toString().padStart(2,'0')}-${d.toString().padStart(2,'0')}`;
                const hebrewMonth = hd.getMonth();
                const hebrewDay = hd.getDate();
                console.log(`\nPage mismatch on ${dateStr} (Hebrew: ${hebrewMonth}/${hebrewDay}):`);
                console.log(`  WASM: ${wasmMapped}`);
                console.log(`  Hebcal: ${hebcalMapped}`);
            }
            expect(wasmMapped).toBe(hebcalMapped);
        }
    });
});
