// Test file for limudim-wasm - Pirkei Avos only
// Run with: bun test

import { pirkei_avos } from "./pkg/limudim_wasm.js";
import { HDate } from "@hebcal/core";
import { pirkeiAvot } from "@hebcal/learning";
import { faker } from "@faker-js/faker";
import { expect, test, describe } from "bun:test";

interface PirkeiAvosSingle {
    type: "Single";
    perek: number;
}

interface PirkeiAvosCombined {
    type: "Combined";
    perek1: number;
    perek2: number;
}

type PirkeiAvosResult = PirkeiAvosSingle | PirkeiAvosCombined;

// Helper to generate random Shabbat dates within a range
function randomShabbatDate(from: string, to: string): Date {
    // Fallback: just generate a date and find the next Shabbat
    const date = faker.date.between({ from, to });
    const hdate = new HDate(date);
    const dayOfWeek = hdate.getDay();
    const daysUntilShabbat = (6 - dayOfWeek + 7) % 7;
    date.setDate(date.getDate() + daysUntilShabbat);
    return date;
}

// Helper to convert hebcal's array format to our format
function hebcalToOurFormat(hebcalResult: number[] | null): PirkeiAvosResult | null {
    if (!hebcalResult || hebcalResult.length === 0) {
        return null;
    }
    
    if (hebcalResult.length === 1) {
        return {
            type: "Single",
            perek: hebcalResult[0],
        };
    }
    
    // Combined chapters
    return {
        type: "Combined",
        perek1: hebcalResult[0],
        perek2: hebcalResult[1],
    };
}

describe("Pirkei Avos - Diaspora", () => {
    test("matches @hebcal/learning implementation for random Shabbat dates 1900-2099 (diaspora)", () => {
        for (let i = 0; i < 200; i++) {
            const date = randomShabbatDate("1900-01-01", "2099-12-31");

            let wasmResult: PirkeiAvosResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = pirkei_avos(date.getFullYear(), date.getMonth() + 1, date.getDate(), false);
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = pirkeiAvot(new HDate(date), false);
            } catch (e) {
                hebcalResult = null;
            }

            const hebcalMapped = hebcalToOurFormat(hebcalResult);

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalMapped === null) continue;
            
            if (!wasmResult || !hebcalMapped) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                const hdate = new HDate(date);
                console.log(`\nNull mismatch on ${dateStr} (Hebrew: ${hdate.toString()}):`);
                console.log(`  WASM: ${wasmResult ? JSON.stringify(wasmResult) : 'null'}`);
                console.log(`  Hebcal: ${hebcalMapped ? JSON.stringify(hebcalMapped) : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalMapped).not.toBeNull();

            // Compare the results
            if (wasmResult!.type !== hebcalMapped!.type) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                const hdate = new HDate(date);
                console.log(`\nType mismatch on ${dateStr} (Hebrew: ${hdate.toString()}):`);
                console.log(`  WASM: ${JSON.stringify(wasmResult)}`);
                console.log(`  Hebcal: ${JSON.stringify(hebcalMapped)}`);
            }
            expect(wasmResult!.type).toBe(hebcalMapped!.type);

            if (wasmResult!.type === "Single" && hebcalMapped!.type === "Single") {
                if (wasmResult.perek !== hebcalMapped.perek) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    const hdate = new HDate(date);
                    console.log(`\nPerek mismatch on ${dateStr} (Hebrew: ${hdate.toString()}):`);
                    console.log(`  WASM: ${wasmResult.perek}`);
                    console.log(`  Hebcal: ${hebcalMapped.perek}`);
                }
                expect(wasmResult.perek).toBe(hebcalMapped.perek);
            } else if (wasmResult!.type === "Combined" && hebcalMapped!.type === "Combined") {
                if (wasmResult.perek1 !== hebcalMapped.perek1 || wasmResult.perek2 !== hebcalMapped.perek2) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    const hdate = new HDate(date);
                    console.log(`\nCombined perek mismatch on ${dateStr} (Hebrew: ${hdate.toString()}):`);
                    console.log(`  WASM: ${wasmResult.perek1}-${wasmResult.perek2}`);
                    console.log(`  Hebcal: ${hebcalMapped.perek1}-${hebcalMapped.perek2}`);
                }
                expect(wasmResult.perek1).toBe(hebcalMapped.perek1);
                expect(wasmResult.perek2).toBe(hebcalMapped.perek2);
            }
        }
    });
});

describe("Pirkei Avos - Israel", () => {
    test("matches @hebcal/learning implementation for random Shabbat dates 1900-2099 (Israel)", () => {
        for (let i = 0; i < 200; i++) {
            const date = randomShabbatDate("1900-01-01", "2099-12-31");

            let wasmResult: PirkeiAvosResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = pirkei_avos(date.getFullYear(), date.getMonth() + 1, date.getDate(), true);
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = pirkeiAvot(new HDate(date), true);
            } catch (e) {
                hebcalResult = null;
            }

            const hebcalMapped = hebcalToOurFormat(hebcalResult);

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalMapped === null) continue;
            
            if (!wasmResult || !hebcalMapped) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                const hdate = new HDate(date);
                console.log(`\nNull mismatch on ${dateStr} (Hebrew: ${hdate.toString()}) [Israel]:`);
                console.log(`  WASM: ${wasmResult ? JSON.stringify(wasmResult) : 'null'}`);
                console.log(`  Hebcal: ${hebcalMapped ? JSON.stringify(hebcalMapped) : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalMapped).not.toBeNull();

            // Compare the results
            if (wasmResult!.type !== hebcalMapped!.type) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                const hdate = new HDate(date);
                console.log(`\nType mismatch on ${dateStr} (Hebrew: ${hdate.toString()}) [Israel]:`);
                console.log(`  WASM: ${JSON.stringify(wasmResult)}`);
                console.log(`  Hebcal: ${JSON.stringify(hebcalMapped)}`);
            }
            expect(wasmResult!.type).toBe(hebcalMapped!.type);

            if (wasmResult!.type === "Single" && hebcalMapped!.type === "Single") {
                if (wasmResult.perek !== hebcalMapped.perek) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    const hdate = new HDate(date);
                    console.log(`\nPerek mismatch on ${dateStr} (Hebrew: ${hdate.toString()}) [Israel]:`);
                    console.log(`  WASM: ${wasmResult.perek}`);
                    console.log(`  Hebcal: ${hebcalMapped.perek}`);
                }
                expect(wasmResult.perek).toBe(hebcalMapped.perek);
            } else if (wasmResult!.type === "Combined" && hebcalMapped!.type === "Combined") {
                if (wasmResult.perek1 !== hebcalMapped.perek1 || wasmResult.perek2 !== hebcalMapped.perek2) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    const hdate = new HDate(date);
                    console.log(`\nCombined perek mismatch on ${dateStr} (Hebrew: ${hdate.toString()}) [Israel]:`);
                    console.log(`  WASM: ${wasmResult.perek1}-${wasmResult.perek2}`);
                    console.log(`  Hebcal: ${hebcalMapped.perek1}-${hebcalMapped.perek2}`);
                }
                expect(wasmResult.perek1).toBe(hebcalMapped.perek1);
                expect(wasmResult.perek2).toBe(hebcalMapped.perek2);
            }
        }
    });
});
