/**
 * Shorthand for `document.querySelector`
 */
export const q = document.querySelector.bind(document);

/**
 * Shorthand for `document.querySelectorAll`
 */
export const qa = document.querySelectorAll.bind(document);

/**
 * Shorthand for `document.addEventListener`
 * 
 * If you want `window`, use `wOn`
 */
export const on = document.addEventListener.bind(document);

/**
 * Shorthand for `document.removeEventListener`
 * 
 * If you want `window`, use `wOff`
 */
export const off = document.removeEventListener.bind(document);

/**
 * Shorthand for `document.addEventListener`
 */
export const dOn = document.addEventListener.bind(document);

/**
 * Shorthand for `document.removeEventListener`
 */
export const dOff = document.removeEventListener.bind(document);

/**
 * Shorthand for `window.addEventListener`
 */
export const wOn = window.addEventListener.bind(window);

/**
 * Shorthand for `window.removeEventListener`
 */
export const wOff = window.removeEventListener.bind(window);

/**
 * Just calls preventDefault on the event.
 * Useful for `thing.addEventListener("something", preventDefault);`
 */
export function preventDefault(ev: Event) {
	ev.preventDefault();
}
