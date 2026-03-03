// Test file for limudim-wasm - Tehillim Monthly only
// Run with: bun test

import { tehillim_monthly } from "./pkg/limudim_wasm.js";
import { HDate } from "@hebcal/core";
import { dailyPsalms } from "@hebcal/learning";
import { faker } from "@faker-js/faker";
import { expect, test, describe } from "bun:test";

interface TehillimResultPsalms {
    type: "Psalms";
    start: number;
    end: number;
}

interface TehillimResultPsalmVerses {
    type: "PsalmVerses";
    psalm: number;
    start_verse: number;
    end_verse: number;
}

type TehillimResult = TehillimResultPsalms | TehillimResultPsalmVerses;

// Helper to generate random dates within a range
function randomDate(from: string, to: string): Date {
    return faker.date.between({ from, to });
}

// Helper to parse hebcal's psalm format which can be like "1-9" or "119:1-30"
function parseHebcalPsalm(psalmBeginEnd: [number | string, number | string]): TehillimResult {
    const [start, end] = psalmBeginEnd;
    
    // Check if it's in verse format (e.g., "119:1")
    if (typeof start === 'string' && start.includes(':')) {
        const [psalm, startVerse] = start.split(':').map(Number);
        const endVerse = typeof end === 'string' && end.includes(':') 
            ? Number(end.split(':')[1]) 
            : Number(end);
        
        return {
            type: "PsalmVerses",
            psalm,
            start_verse: startVerse,
            end_verse: endVerse,
        };
    }
    
    // Regular psalm range
    return {
        type: "Psalms",
        start: Number(start),
        end: Number(end),
    };
}

describe("Tehillim Monthly", () => {
    test("matches @hebcal/learning implementation for random dates 1900-2099", () => {
        for (let i = 0; i < 200; i++) {
            const date = randomDate("1900-01-01", "2099-12-31");

            let wasmResult: TehillimResult | null = null;
            let hebcalResult: any = null;

            try {
                wasmResult = tehillim_monthly(date.getFullYear(), date.getMonth() + 1, date.getDate());
            } catch (e) {
                wasmResult = null;
            }

            try {
                hebcalResult = dailyPsalms(new HDate(date));
            } catch (e) {
                hebcalResult = null;
            }

            // Both should agree on whether result exists
            if (wasmResult === null && hebcalResult === null) continue;
            
            if (!wasmResult || !hebcalResult) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nNull mismatch on ${dateStr}:`);
                console.log(`  WASM: ${wasmResult ? JSON.stringify(wasmResult) : 'null'}`);
                console.log(`  Hebcal: ${hebcalResult ? JSON.stringify(hebcalResult) : 'null'}`);
            }
            expect(wasmResult).not.toBeNull();
            expect(hebcalResult).not.toBeNull();

            const hebcalParsed = parseHebcalPsalm(hebcalResult);

            // Compare the results
            if (wasmResult!.type !== hebcalParsed.type) {
                const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                console.log(`\nType mismatch on ${dateStr}:`);
                console.log(`  WASM: ${JSON.stringify(wasmResult)}`);
                console.log(`  Hebcal: ${JSON.stringify(hebcalParsed)}`);
            }
            expect(wasmResult!.type).toBe(hebcalParsed.type);

            if (wasmResult!.type === "Psalms" && hebcalParsed.type === "Psalms") {
                if (wasmResult.start !== hebcalParsed.start || wasmResult.end !== hebcalParsed.end) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    console.log(`\nPsalm mismatch on ${dateStr}:`);
                    console.log(`  WASM: ${wasmResult.start}-${wasmResult.end}`);
                    console.log(`  Hebcal: ${hebcalParsed.start}-${hebcalParsed.end}`);
                }
                expect(wasmResult.start).toBe(hebcalParsed.start);
                expect(wasmResult.end).toBe(hebcalParsed.end);
            } else if (wasmResult!.type === "PsalmVerses" && hebcalParsed.type === "PsalmVerses") {
                if (wasmResult.psalm !== hebcalParsed.psalm || 
                    wasmResult.start_verse !== hebcalParsed.start_verse || 
                    wasmResult.end_verse !== hebcalParsed.end_verse) {
                    const dateStr = `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2,'0')}-${date.getDate().toString().padStart(2,'0')}`;
                    console.log(`\nPsalm verses mismatch on ${dateStr}:`);
                    console.log(`  WASM: ${wasmResult.psalm}:${wasmResult.start_verse}-${wasmResult.end_verse}`);
                    console.log(`  Hebcal: ${hebcalParsed.psalm}:${hebcalParsed.start_verse}-${hebcalParsed.end_verse}`);
                }
                expect(wasmResult.psalm).toBe(hebcalParsed.psalm);
                expect(wasmResult.start_verse).toBe(hebcalParsed.start_verse);
                expect(wasmResult.end_verse).toBe(hebcalParsed.end_verse);
            }
        }
    });
});
