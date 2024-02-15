/**
 * Checks to see whether or not the specified element's bounding box is in the viewport
 * @param elem The element to check
 * @param completely if true, this function will only return true if the element is completely within the viewport.
 * Otherwise, it will also return true if the element is partially in the viewport
 * @returns Whether or not the element is in the viewport
 */
export function isElementInViewport(elem: Element, completely: boolean = false): boolean {
	const rect = elem.getBoundingClientRect();

	// This is honestly the strangest code I've ever written
	return completely ? (
		rect.top >= 0 &&
		rect.left >= 0 &&
		rect.bottom <= document.documentElement.clientHeight &&
		rect.right <= document.documentElement.clientWidth
	) : (
		(
			(
				rect.top >= 0 &&
				rect.top <= document.documentElement.clientHeight
			) || (
				rect.bottom >= 0 &&
				rect.bottom <= document.documentElement.clientHeight
			)
		) && (
			(
				rect.left >= 0 &&
				rect.left <= document.documentElement.clientWidth
			) || (
				rect.right >= 0 &&
				rect.right <= document.documentElement.clientWidth
			)
		)
	);
}
