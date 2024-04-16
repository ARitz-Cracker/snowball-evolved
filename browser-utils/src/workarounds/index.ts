export function needsRangeInputFillWorkaround() {
	// At the time of writing, only Firefox provides a way to style the filled-in area of a range input.
	// Right now the workaround is to have all rage inputs have a --range-workaround-fill-amount CSS variable which is
	// then set to a value between 0 and 1 depending on where the slider is. 
	return !CSS.supports("selector(input::-moz-range-progress)");
}
export async function applyRangeInputFillWorkaround() {
	if (needsRangeInputFillWorkaround()) {
		// The use of Function.prototype helps pevent tree-shaking
		Function.prototype(await import("./range-input-fill.js"));
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
export async function applyCustomElementsWorkaround() {
	if (needsCustomElementsWorkaround()) {
		// The use of Function.prototype helps pevent tree-shaking
		Function.prototype(await import("@ungap/custom-elements" as any));
	}
}

export async function applyAllWorkarounds() {
	await Promise.all([
		applyCustomElementsWorkaround(),
		applyRangeInputFillWorkaround()
	]);
}
