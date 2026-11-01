import { describe, expect, test } from "vitest";
import { chanceText, formatValue, formulaText, headingText, ordinal, ruledOutText } from "./why-text.ts";

describe("why-text", () => {
  test("ordinals", () => {
    expect([1, 2, 3, 4, 11, 12, 13, 21, 22, 41, 101, 111, 384].map(ordinal)).toEqual([
      "1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "22nd", "41st", "101st", "111th", "384th",
    ]);
  });

  test("values keep two decimals, and small ones two significant figures", () => {
    expect(formatValue(1.2)).toBe("1.2");
    expect(formatValue(1.0605)).toBe("1.06");
    expect(formatValue(0.5)).toBe("0.5");
    expect(formatValue(0.05)).toBe("0.05");
    expect(formatValue(0.0025)).toBe("0.0025");
    expect(formatValue(0.000123)).toBe("0.00012");
    expect(formatValue(0)).toBe("0");
  });

  test("chance has one decimal and a % sign, <0.1% below 0.05%", () => {
    expect(chanceText(0.589)).toBe("58.9%");
    expect(chanceText(1)).toBe("100.0%");
    expect(chanceText(0.0004)).toBe("<0.1%");
    expect(chanceText(0)).toBe("0%");
  });

  test("the heading, the ruled-out count and the formula line", () => {
    expect(headingText("═", 48213, 14, 2, 41, 384)).toBe("Why ═ is here · seed 48213 · col 14, row 2 · drawn 41st of 384");
    expect(ruledOutText(27)).toBe("+ 27 ruled out before the draw");
    expect(formulaText(1.74, "═", true)).toBe(
      "score = tile weight × one pair factor per counted neighbour (side-pair base × exceptions × noise; tile weights left out). " +
        "chance = score ÷ 1.74, the sum of the scores. " +
        "Neighbours that came later were drawn to fit ═, so they don't count here.",
    );
    expect(formulaText(1.74, "═", false)).not.toContain("came later");
  });
});
