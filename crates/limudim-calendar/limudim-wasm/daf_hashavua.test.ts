// Test file for limudim-wasm - Daf Hashavua Bavli only
// Run with: bun test

import { daf_hashavua_bavli } from "./pkg/limudim_wasm.js";
import { HDate } from "@hebcal/core";
import { dafWeekly } from "@hebcal/learning";
import { faker } from "@faker-js/faker";
import { expect, test, describe } from "bun:test";

// Mapping from our tractate names to hebcal's tractate names (Bavli)
const TRACTATE_MAP: Record<string, string> = {
    Berachos: "Berachot",
    Shabbos: "Shabbat",
    Eruvin: "Eruvin",
    Pesachim: "Pesachim",
    Shekalim: "Shekalim",
    Yoma: "Yoma",
    Sukkah: "Sukkah",
    Beitzah: "Beitzah",
    RoshHashanah: "Rosh Hashana",
    Taanis: "Taanit",
    Megillah: "Megillah",
    MoedKatan: "Moed Katan",
    Chagigah: "Chagigah",
    Yevamos: "Yevamot",
    Kesubos: "Ketubot",
    Nedarim: "Nedarim",
    Nazir: "Nazir",
    Sotah: "Sotah",
    Gitin: "Gittin",
    Kiddushin: "Kiddushin",
    BavaKamma: "Baba Kamma",
    BavaMetzia: "Baba Metzia",
    BavaBasra: "Baba Batra",
    Sanhedrin: "Sanhedrin",
    Makkos: "Makkot",
    Shevuos: "Shevuot",
    AvodahZarah: "Avodah Zarah",
    Horiyos: "Horayot",
    Zevachim: "Zevachim",
    Menachos: "Menachot",
    Chullin: "Chullin",
    Bechoros: "Bekhorot",
    Bechorot: "Bekhorot", // Hebcal sometimes returns Bechorot
    Bekhorot: "Bekhorot", // Canonical
    Arachin: "Arakhin",
    Temurah: "Temurah",
    Kerisos: "Keritot",
    Meilah: "Meilah",
    Kinnim: "Kinnim",
    Tamid: "Tamid",
    Midos: "Middot",
    Midot: "Middot", // Hebcal sometimes returns Midot
    Middot: "Middot", // Canonical
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

describe("Daf Hashavua Bavli", () => {
    test("matches @hebcal/learning implementation for random dates 1900-2099", () => {
        for (let i = 0; i < 200; i++) {
            const date = randomDate("1900-01-01", "2099-12-31");

            let wasmResult: DafResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = daf_hashavua_bavli(date.getFullYear(), date.getMonth() + 1, date.getDate());
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = dafWeekly(new HDate(date));
            } catch (e) {
                hebcalResult = null;
            }

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalResult === null) continue;
            
            if (!wasmResult || !hebcalResult) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nNull mismatch on ${dateStr}:`);
                console.log(`  WASM: ${wasmResult ? `${wasmResult.tractate}:${wasmResult.page}` : 'null'}`);
                console.log(`  Hebcal: ${hebcalResult ? `${hebcalResult.getName()}:${hebcalResult.getBlatt()}` : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalResult).not.toBeNull();

            const wasmMapped = `${mapTractate(wasmResult!.tractate)}:${wasmResult!.page}`;
            const hebcalMapped = `${mapTractate(hebcalResult.getName())}:${hebcalResult.getBlatt()}`;

            if (wasmMapped !== hebcalMapped) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nPage mismatch on ${dateStr}:`);
                console.log(`  WASM: ${wasmMapped}`);
                console.log(`  Hebcal: ${hebcalMapped}`);
            }
            expect(wasmMapped).toBe(hebcalMapped);
        }
    });
});
