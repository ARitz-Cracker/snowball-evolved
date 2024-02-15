/**
 * See https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/elements#value
 */
export type HTMLFormControlsElement =
	HTMLButtonElement |
	HTMLFieldSetElement |
	HTMLInputElement |
	HTMLObjectElement |
	HTMLOutputElement |
	HTMLSelectElement |
	HTMLTextAreaElement;

/**
 * Sets all form inputs to `disabled` for the specified form
 * @param form The form element which to disable all contained input elements
 * @param exemptions Things which you don't want this function to touch
 */
export function disableFormInputs(form: HTMLFormElement, exemptions: HTMLFormControlsElement[] = []) {
	const exemptSet = new Set(exemptions);
	for (let i = 0; i < form.elements.length; i++) {
		const input = form.elements[i] as HTMLFormControlsElement;
		if (exemptSet.has(input) || !("disabled" in input)) {
			continue;
		}
		input.disabled = true;
	}
	  
}

/**
 * Sets all form inputs to `disabled` for the specified form
 * @param form The form element which to enable all contained input elements
 * @param exemptions Things which you don't want this function to touch
 */
export function enableFormInputs(form: HTMLFormElement, exemptions: HTMLFormControlsElement[] = []) {
	const exemptSet = new Set(exemptions);
	for (let i = 0; i < form.elements.length; i++) {
		const input = form.elements[i] as HTMLFormControlsElement;
		if (exemptSet.has(input) || !("disabled" in input)) {
			continue;
		}
		input.disabled = false;
	}
}

/**
 * Takes in a form element or submit event and returns a mapping between the input's name and their values
 * 
 * If using a `SubmitEvent`, the `name` and `value` of the `HTMLButtonElement` used to submit the form will be
 * included.
 * 
 * @param source `HTMLFormElement` or `SubmitEvent` to get the values from
 * @returns a mapping between the input's name and their values. The type of value is determined as so:
 * * `<input type="checkbox">`: `boolean` or `null` if indeterminate
 * * `<input type="datetime-local">`: `Date` or `null` if none is entered
 * * `<input type="file">`: `FileList`
 * * `<input type="number">`: `number`
 * * `<input type="range">`: `number`
 * * `<input type="radio">`: `string` - The value of the selected radio button. If none are selected, this will be an
 * empty string. This reflects the behaviour of `RadioNodeList`.
 * * All other inputs: `string`
 */
export function normalizeFormValues(
	source: HTMLFormElement | SubmitEvent
): {[inputName: string]: string | number | boolean | Date | FileList | null | undefined} {
	const result: any = {};
	const [formElement, submitter] = (() => {
		if (source instanceof HTMLFormElement) {
			return [source, null];
		}
		return [source.target as HTMLFormElement, source.submitter];
	})();
	const uncheckedRadioNames: Set<string> = new Set();
	for (let i = 0; i < formElement.elements.length; i += 1) {
		const formControl = formElement.elements[i];
		if (formControl instanceof HTMLButtonElement) {
			if (formControl == submitter) {
				if (formControl.name) {
					result[formControl.name] = formControl.value;
				}
			}
		}else if (formControl instanceof HTMLInputElement) {
			switch(formControl.type) {
				case "checkbox": {
					if (formControl.indeterminate) {
						result[formControl.name] = null;
					}else{
						result[formControl.name] = formControl.checked;
					}
					break;
				}
				case "datetime-local": {
					result[formControl.name] = formControl.valueAsDate;
					break;
				}
				case "file": {
					result[formControl.name] = formControl.files;
					break;
				}
				case "number":
				case "range": {
					result[formControl.name] = formControl.valueAsNumber;
					break;
				}
				case "radio": {
					if (formControl.checked) {
						result[formControl.name] = formControl.value;
					}else{
						uncheckedRadioNames.add(formControl.name);
					}
					break;
				}
				default:
					result[formControl.name] = formControl.value;
			}
		}else if (
			formControl instanceof HTMLOutputElement ||
			formControl instanceof HTMLSelectElement ||
			formControl instanceof HTMLTextAreaElement
		) {
			result[formControl.name] = formControl.value;
		}
	}
	uncheckedRadioNames.forEach(name => {
		// `RadioNodeList` returns a "" if no radio buttons are selected. Might as well reflect that behaviour!
		if (!result[name]) {
			result[name] = "";
		}
	})
	return result;
}
