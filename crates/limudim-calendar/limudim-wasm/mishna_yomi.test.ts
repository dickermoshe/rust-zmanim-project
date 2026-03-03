// Test file for limudim-wasm - Mishna Yomi only
// Run with: bun test

import { mishna_yomis } from "./pkg/limudim_wasm.js";
import { HDate } from "@hebcal/core";
import { MishnaYomiIndex } from "@hebcal/learning";
import { faker } from "@faker-js/faker";
import { expect, test, describe } from "bun:test";

// Mapping from our tractate names to hebcal's tractate names
const TRACTATE_MAP: Record<string, string> = {
    Berachos: "Berakhot",
    Peah: "Peah",
    Demai: "Demai",
    Kilayim: "Kilayim",
    Sheviis: "Sheviit",
    Terumos: "Terumot",
    Maasros: "Maasrot",
    MaaserSheni: "Maaser Sheni",
    Chalah: "Challah",
    Orlah: "Orlah",
    Bikurim: "Bikkurim",
    Shabbos: "Shabbat",
    Eruvin: "Eruvin",
    Pesachim: "Pesachim",
    Shekalim: "Shekalim",
    Yoma: "Yoma",
    Sukkah: "Sukkah",
    Beitzah: "Beitzah",
    RoshHashanah: "Rosh Hashanah",
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
    BavaKamma: "Bava Kamma",
    BavaMetzia: "Bava Metzia",
    BavaBasra: "Bava Batra",
    Sanhedrin: "Sanhedrin",
    Makkos: "Makkot",
    Shevuos: "Shevuot",
    Eduyos: "Eduyot",
    AvodahZarah: "Avodah Zarah",
    Avos: "Avot",
    Horiyos: "Horayot",
    Zevachim: "Zevachim",
    Menachos: "Menachot",
    Chullin: "Chullin",
    Bechoros: "Bekhorot",
    Arachin: "Arakhin",
    Temurah: "Temurah",
    Kerisos: "Keritot",
    Meilah: "Meilah",
    Tamid: "Tamid",
    Midos: "Middot",
    Kinnim: "Kinnim",
    Keilim: "Kelim",
    Ohalos: "Oholot",
    Negaim: "Negaim",
    Parah: "Parah",
    Taharos: "Tahorot",
    Mikvaos: "Mikvaot",
    Niddah: "Niddah",
    Machshirin: "Makhshirin",
    Zavim: "Zavim",
    TevulYom: "Tevul Yom",
    Yadayim: "Yadayim",
    Uktzin: "Oktzin",
};

function mapTractate(ourTractate: string): string {
    return TRACTATE_MAP[ourTractate] || ourTractate;
}

interface MishnaResult {
    start: {
        tractate: string;
        chapter: number;
        mishna: number;
    };
    end: {
        tractate: string;
        chapter: number;
        mishna: number;
    };
}

// Helper to generate random dates within a range
function randomDate(from: string, to: string): Date {
    return faker.date.between({ from, to });
}

describe("Mishna Yomi", () => {
    test("matches @hebcal/learning implementation for random dates 1947-2099", () => {
        const mishnaYomiIndex = new MishnaYomiIndex();
        
        for (let i = 0; i < 200; i++) {
            const date = randomDate("1947-05-20", "2099-12-31");

            let wasmResult: MishnaResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = mishna_yomis(date.getFullYear(), date.getMonth() + 1, date.getDate());
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = mishnaYomiIndex.lookup(new HDate(date));
            } catch (e) {
                hebcalResult = null;
            }

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalResult === null) continue;
            
            if (!wasmResult || !hebcalResult || hebcalResult.length !== 2) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nNull mismatch on ${dateStr}:`);
                console.log(`  WASM: ${wasmResult ? `${wasmResult.start.tractate} ${wasmResult.start.chapter}:${wasmResult.start.mishna} - ${wasmResult.end.tractate} ${wasmResult.end.chapter}:${wasmResult.end.mishna}` : 'null'}`);
                console.log(`  Hebcal: ${hebcalResult ? hebcalResult.map((m: any) => `${m.k} ${m.v}`).join(' - ') : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalResult).not.toBeNull();
            expect(hebcalResult.length).toBe(2);

            // Compare start mishna
            const wasmStartMapped = `${mapTractate(wasmResult!.start.tractate)}:${wasmResult!.start.chapter}:${wasmResult!.start.mishna}`;
            const hebcalStartMapped = `${hebcalResult[0].k}:${hebcalResult[0].v.replace(':', ':')}`;
            
            // Compare end mishna
            const wasmEndMapped = `${mapTractate(wasmResult!.end.tractate)}:${wasmResult!.end.chapter}:${wasmResult!.end.mishna}`;
            const hebcalEndMapped = `${hebcalResult[1].k}:${hebcalResult[1].v.replace(':', ':')}`;

            if (wasmStartMapped !== hebcalStartMapped || wasmEndMapped !== hebcalEndMapped) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nMishna mismatch on ${dateStr}:`);
                console.log(`  WASM: ${wasmStartMapped} - ${wasmEndMapped}`);
                console.log(`  Hebcal: ${hebcalStartMapped} - ${hebcalEndMapped}`);
            }
            
            expect(wasmStartMapped).toBe(hebcalStartMapped);
            expect(wasmEndMapped).toBe(hebcalEndMapped);
        }
    });
});
