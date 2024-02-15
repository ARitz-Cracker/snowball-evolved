
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
