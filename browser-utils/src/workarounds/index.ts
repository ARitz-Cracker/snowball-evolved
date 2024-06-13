export function needsRangeInputFillWorkaround() {
	// At the time of writing, only Firefox provides a way to style the filled-in area of a range input.
	// Right now the workaround is to have all rage inputs have a --range-workaround-fill-amount CSS variable which is
	// then set to a value between 0 and 1 depending on where the slider is. 
	return !CSS.supports("selector(input::-moz-range-progress)");
}

let importRangeInputFillWorkaroundPromise: Promise<typeof import("./range-input-fill.js")> | undefined;
/**
 * Applies a workaround for styling the fill of `<input type="range">` elements.
 * 
 * At the time of writing, only Firefox provides a way to style the filled-in area of a range input via the
 * `input::-moz-range-progress` psudo element. If that psudo element is found to not exist, then a
 * `--range-workaround-fill-amount` CSS variable will be set to a value between 0 and 1 based on the following formula
 * `(value - min) / (max - min)`. This, for example, can be used to set the width of the `::before` or `::after` psudo
 * elements.
 * 
 * @param always apply the `--range-workaround-fill-amount` regardless of whether or not the
 * `input::-moz-range-progress` psudo-element exists.
 */
export async function applyRangeInputFillWorkaround(always?: boolean) {
	if (always || needsRangeInputFillWorkaround()) {
		if (!importRangeInputFillWorkaroundPromise) {
			importRangeInputFillWorkaroundPromise = import("./range-input-fill.js");
		}
		await importRangeInputFillWorkaroundPromise;
	}	
}

export function needsCustomElementsWorkaround() {
	try {
		const newElemName = "test-button-" + Date.now().toString(36);
		class HTMLTestButton extends HTMLButtonElement {};
		customElements.define(newElemName, HTMLTestButton, { extends: "button" });
		const newBtn = document.createElement("button", { is: newElemName });
		return !(newBtn instanceof HTMLButtonElement && newBtn instanceof HTMLTestButton);
	}catch(ex: any) {
		return true;
	}
}

let importCustomElementsWorkaroundPromise: Promise<any> | undefined;
/**
 * Imports the `@ungap/custom-elements` package if the browser does not support creating classes which extend built-in
 * HTML elements.
 */
export async function applyCustomElementsWorkaround() {
	if (needsCustomElementsWorkaround()) {
		// The use of Function.prototype helps pevent tree-shaking
		Function.prototype(await import("@ungap/custom-elements" as any));
		if (!importCustomElementsWorkaroundPromise) {
			importCustomElementsWorkaroundPromise = import("@ungap/custom-elements" as any);
		}
		await importCustomElementsWorkaroundPromise;
	}
}

export async function applyAllWorkarounds() {
	await Promise.all([
		applyCustomElementsWorkaround(),
		applyRangeInputFillWorkaround()
	]);
}
