
/**
 * Parses the given string as a CSS time value and returns the result in milliseconds.
 * 
 * See https://developer.mozilla.org/en-US/docs/Web/CSS/time
 * @param time A CSS Time value. If `""`, `null`, or `undefined` is given, `0` is returned.
 * @returns The result in milliseconds, or `NaN` if the input was invalid
 */
export function parseCSSTime(time: string | null | undefined = "0s"): number {
	if(!time){
		return 0;
	}
	let [_, sign, number, unit] = (() => {
		const result = time.match(/^\s*([+-]?)([0-9.]+)(s|ms)\s*$/i);
		if (result == null) {
			return ["", "", "NaN", "ms"]
		}
		return result;
	})();
	let result = Number(number);
	if(unit.toLowerCase() === "s"){
		result *= 1000;
	}
	result = Math.ceil(result);
	if(sign === "-"){
		result *= -1;
	}
	return result;
}

// The cubic bezier function implementation has been taken from "framer-motion" by Matt Perry.
// https://github.com/framer/motion/blob/ea49a4dc0fbf1662b89abb277e2020d459026b54/packages/framer-motion/src/easing/cubic-bezier.ts
// https://github.com/framer/motion/blob/ea49a4dc0fbf1662b89abb277e2020d459026b54/LICENSE.md
// "framer-motion" uses code adapted code from "bezier-easing" by Gaëtan Renaudeau.
// https://github.com/gre/bezier-easing/
// https://github.com/gre/bezier-easing/blob/7785b0b63acd6fefb240607f169f037b8add26ad/LICENSE


// Returns x(t) given t, x1, and x2, or y(t) given t, y1, and y2.
function calcBezier(t: number, a1: number, a2: number) {
	return (
		((1 - 3 * a2 + 3 * a1) * t + (3 * a2 - 6 * a1)) * t + 3 * a1
	) * t;
}

const subdivisionPrecision = 0.0000001
const subdivisionMaxIterations = 12

function binarySubdivide(
	x: number,
	lowerBound: number,
	upperBound: number,
	mX1: number,
	mX2: number
) {
    let currentX: number
    let currentT: number
    let i: number = 0
    do {
        currentT = lowerBound + (upperBound - lowerBound) / 2.0
        currentX = calcBezier(currentT, mX1, mX2) - x
        if (currentX > 0.0) {
            upperBound = currentT
        } else {
            lowerBound = currentT
        }
    } while (
        Math.abs(currentX) > subdivisionPrecision &&
        ++i < subdivisionMaxIterations
    )
    return currentT
}

function clampedIdentity(t: number) {
	if (t <= 0) {
		return 0;
	}
	if (t >= 1) {
		return 1;
	}
	return t;
}

export function newCubicBezierFunction(
    mX1: number,
    mY1: number,
    mX2: number,
    mY2: number
): (t: number) => number {
	// If this is a linear gradient, return linear easing
	if (mX1 === mY1 && mX2 === mY2) {
		return clampedIdentity;
	};
	const getTForX = (aX: number) => binarySubdivide(aX, 0, 1, mX1, mX2)
	return (t: number) => {
		// No need to do expensive math or return values that don't make sense
		if (t <= 0) {
			return 0;
		}
		if (t >= 1) {
			return 1;
		}
		return calcBezier(getTForX(t), mY1, mY2);
	}
}

// export type CSSJumpTerm = "jump-start" | "jump-end" | "jump-none" | "jump-both" | "start" | "end";
export function newStepsFunction(steps: number, direction?: string): ((t: number) => number) | undefined {
	if (isNaN(steps) || steps <= 0) {
		return undefined;
	}
	switch (direction) {
		case "start":
		case "jump-start":
			return (t: number) => {
				if (t <= 0) {
					return 0;
				}
				if (t >= 1) {
					return 1;
				}
				return Math.ceil(t * steps) / steps
			};
		case "end":
		case "jump-end":
			return (t: number) => {
				if (t <= 0) {
					return 0;
				}
				if (t >= 1) {
					return 1;
				}
				return Math.floor(t * steps) / steps
			};
		case "jump-none":
			return (t: number) => {
				if (t <= 0) {
					return 0;
				}
				if (t >= 1) {
					return 1;
				}
				// I really just threw stuff at the wall until something stuck
				return Math.floor(t * steps) / (steps - 1);
			};
		case "jump-both":
			return (t: number) => {
				if (t <= 0) {
					return 0;
				}
				if (t >= 1) {
					return 1;
				}
				// Ditto, after spending too much time on the MDN playground
				return Math.min((Math.floor(t * (steps)) / (steps + 1)) + 1 / (steps + 1), 1)
			};
		default:
			return undefined;
	}
}

function fillInImplicitStops(
	args: ([number] | [number, number | undefined])[]
): asserts args is [number, number][] {
	if (!args.length) {
		return;
	}
	if (args[0][1] == undefined) {
		args[0][1] = 0;
	}
	if (args[args.length - 1][1] == undefined) {
		args[args.length - 1][1] = 1;
	}
	for (let i = 0; i < args.length; i += 1) {
		if (args[i][1] == undefined) {
			const startVal = args[i - 1][1]!;
			// I, once again, miss rust expressions. ;_;
			const [endVal, endIndex] = (() => {
				for (let ii = i; ii < args.length; ii += 1) {
					if (args[ii][1] != undefined) {
						return [args[ii][1]!, ii];
					}
				}
				// This should be unreachable as the last item is guaranteed to be [number, number]
				throw new Error("2+2=5");
			})();
			const stepCount = endIndex - i;
			for (let ii = i; ii < endIndex; ii += 1) {
				const fractional = (ii - i + 1) / (stepCount + 1);
				args[ii][1] = startVal * (1 - fractional) + endVal * fractional;
			}
			i += stepCount;
		}
	}
}

export function newPiecewiseLinearFunction(args: ([number] | [number, number | undefined])[]) {
	if (!args.length) {
		return clampedIdentity;
	}
	fillInImplicitStops(args);
	return (t: number) => {
		if (t < 0) {
			t = 0;
		} else if (t > 1) {
			t = 1;
		}
		// Linear search is fine... though it may be worth benchmarking against binary search at some point
		for (let i = 0; i < args.length; i += 1) {
			if (t < args[i][1]) {
				const startVal = args[i - 1][0] ?? 0;
				const endVal = args[i][0];
				const fractional = (t - (args[i - 1][1] ?? 0)) / (args[i][1] - (args[i - 1][1] ?? 0));
				return startVal * (1 - fractional) + endVal * fractional;
			}
		}
		return args[args.length - 1][1];
	};
}

const COMMON_CSS_TIMING_FUNCS = {
	"ease": newCubicBezierFunction(0.25, 0.1, 0.25, 1.0),
	"linear": clampedIdentity,
	"ease-in": newCubicBezierFunction(0.42, 0, 1.0, 1.0),
	"ease-out": newCubicBezierFunction(0, 0, 0.58, 1.0),
	"ease-in-out": newCubicBezierFunction(0.42, 0, 0.58, 1.0),
	"step-start": (t: number) => t > 0 ? 1 : 0,
	"step-end": (t: number) => t < 1 ? 0 : 1
}

/**
 * Parses a [CSS easing function](https://developer.mozilla.org/en-US/docs/Web/CSS/easing-function) and returns a
 * function whichs takes a value between 0 and 1, and returns a value between 0 and 1. If the input to the returned
 * function exceeds those bounds, it will be clamped.
 * 
 * If the inputted string is not a valid easing function, (as defined during April 2024) then `undefined` is returned.
 * @param func [CSS easing function](https://developer.mozilla.org/en-US/docs/Web/CSS/easing-function)
 * @returns the resulting easing function, or `undefined` if the string provided is invalid.
 */
export function parseCSSTimingFunction(func: string): ((t: number) => number) | undefined {
	if (func in COMMON_CSS_TIMING_FUNCS) {
		return COMMON_CSS_TIMING_FUNCS[func as keyof typeof COMMON_CSS_TIMING_FUNCS];
	}
	const [_, p1, p2, p3, p4] = (() => {
		const result = func.match(/^\s*cubic-bezier\(\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*,\s*([0-9.]+)\s*\)\s*;?\s*$/i);
		if (result == null) {
			return [NaN, NaN, NaN, NaN, NaN];
		}
		return result.map(Number);
	})();
	if (!isNaN(p1) && !isNaN(p2) && !isNaN(p3) && !isNaN(p4)) {
		return newCubicBezierFunction(p1, p2, p3, p4);
	}
	let parseResult = func.match(/^\s*steps\(\s*([0-9.]+)\s*,\s*(.+)\s*\)\s*;?\s*$/);
	if (parseResult != null) {
		return newStepsFunction(Number(parseResult[1]), parseResult[2]);
	}
	parseResult = func.match(/^\s*linear\(\s*([0-9.,% ]+\s*)+\s*\)\s*;?\s*$/);
	if (parseResult == null) {
		return;
	}
	const piecewiseArgs: [number, number | undefined][] = [];
	const argStrs = parseResult[1].split(",");
	for (let i = 0; i < argStrs.length; i += 1) {
		const argStr = argStrs[i];
		const parseArgResult = argStr.match(/^\s*([0-9.]+)\s*(?:\s+([0-9.]+)%\s*)?(?:\s+([0-9.]+)%\s*)?$/);
		if (parseArgResult == null) {
			return;
		}
		const argPart1 = Number(parseArgResult[1]);
		if (isNaN(argPart1)) {
			return;
		}
		const argPart2 = parseArgResult[2] ? Number(parseArgResult[2]) / 100 : undefined;
		if (argPart2 != undefined && isNaN(argPart1)) {
			return;
		}
		piecewiseArgs.push([argPart1, argPart2]);
		if (parseArgResult[3]) {
			const argPart3 = Number(parseArgResult[3]) / 100;
			if (isNaN(argPart3)) {
				return;
			}
			piecewiseArgs.push([argPart1, argPart3]);
		}
	}
	return newPiecewiseLinearFunction(piecewiseArgs);
}
