// Get the original getters/setters for the input element
const originalInputDescriptor = Object.getOwnPropertyDescriptor(
	HTMLInputElement.prototype,
	"valueAsNumber"
)!;

function updateCSSVariable(elem: HTMLInputElement, curVal: number = elem.valueAsNumber){
	if(elem.type !== "range"){
		return;
	}
	// The default min/max values replicates built-in behaviour
	const minVal = isNaN(elem.min as any) ? 0 : Number(elem.min);
	const maxVal = isNaN(elem.max as any) ? 100 : Number(elem.max);
	elem.style.setProperty(
		'--range-workaround-fill-amount',
		((curVal - minVal) / (maxVal - minVal)) + ""
	);
}
// Unique symbol to check if we already patched
const rangeProgressWorkaroundApplied = Symbol("rangeProgressWorkaroundApplied");

// Patches the element so that changes to .value and .valueAsNumber calls updateCSSVariable 
function patchRangeInput(elem: HTMLInputElement){
	if((elem as any)[rangeProgressWorkaroundApplied]){
		if (elem.type != "range") {
			// This element isn't a range input anymore, remove our setters.
			delete (elem as any).value;
			delete (elem as any).valueAsNumber;
			// This element may turn back into a range input in the future...
			delete (elem as any)[rangeProgressWorkaroundApplied];
		}
		return;
	}
	if (elem.type != "range") {
		return;
	}
	function setterValueCallback(val: any) {
		val = Number(val);
		if(isNaN(val)){
			// The default min/max values replicates built-in behaviour
			const minVal = isNaN(elem.min as any) ? 0 : Number(elem.min);
			const maxVal = isNaN(elem.max as any) ? 100 : Number(elem.max);
			// Setting the value to 50% on an invalid value replicates built-in behaviour
			val = Math.round((minVal + maxVal) / 2);
		}
		updateCSSVariable(elem, val);
		(originalInputDescriptor.set as any).call(elem, val);
	}
	Object.defineProperty(elem, "value", {
		set: setterValueCallback,
		get:() => {
			return String((originalInputDescriptor.get as any).call(elem));
		}
	});
	Object.defineProperty(elem, "valueAsNumber", {
		set: setterValueCallback,
		get:() => {
			return (originalInputDescriptor.get as any).call(elem);
		}
	});
	(elem as any)[rangeProgressWorkaroundApplied] = true;
	updateCSSVariable(elem);
}
function updateCSSVariableOnTarget(ev: Event) {
	updateCSSVariable(ev.target as HTMLInputElement);
}
document.addEventListener("input", updateCSSVariableOnTarget, {passive: true});
document.addEventListener("change", updateCSSVariableOnTarget, {passive: true});

const newRangeInputObserver = new MutationObserver((records) => {
	for (const record of records) {
		for (const addedNode of record.addedNodes) {
			if (addedNode instanceof HTMLInputElement) {
				patchRangeInput(addedNode);
			}
		}
		if (record.target instanceof HTMLInputElement) {
			patchRangeInput(record.target);
		}
	}
});

function onDocumentLoaded() {
	(
		document.querySelectorAll("input[type=\"range\"]") as NodeListOf<HTMLInputElement>
	).forEach(elem => {
		patchRangeInput(elem);
	})
	newRangeInputObserver.observe(document.body, {childList: true, subtree: true, attributeFilter: ["type"]});
}
if (document.readyState == "loading") {
	document.addEventListener("DOMContentLoaded", onDocumentLoaded);
} else {
	onDocumentLoaded();
}
