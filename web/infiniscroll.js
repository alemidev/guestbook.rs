/**
 * @typedef GuestbookPage
 * @type {Object}
 * @property {number} id - unique page id
 * @property {string} author - name of page author
 * @property {string} body - main page body
 * @property {string|undefined} contact - optional page author's contact
 * @property {string|undefined} url - optional url to author's contact, if either mail or http link
 * @property {string} avatar - unique hash (based on contact) to use for libravatar img href
 * @property {Date} date - time of creation for page
 * @property
 */

/**
 * Hook infinite scroll callback with element builder
 *
 * @param {(data:GuestbookPage)=>string} builder - function invoked to generate new elements, must return HTML string
 * @param {Object} [opt] - configuration options for infinite scroll behavior
 * @param {number} [opt.threshold] - from 0 to 1, percentage of scroll at which new fetch triggers (default 0.9)
 * @param {number} [opt.debounce] - how many milliseconds to wait before processing new scroll events (default 250)
 * @param {string} [opt.container_id] - html id of container element to inject new values into (default '#container')
 * @param {string} [opt.api_url] - url for api calls (default '/api')
 * @param {string} [opt.offset_arg] - how the offset/page argument is called in the api backend (default 'offset')
 * @param {boolean} [opt.prefetch] - load first page immediately, before any scroll event is received (default true)
 * @param {boolean} [opt.free_index] - wether backend api allows free indexing (?offset=17) or expects pagination (?page=1) (default true)
 *
 * @example
 *   hookInfiniteScroll(
 *     (data) => { return `<li><b>${data.author}</b>: ${data.body}</li>` },
 *     {threshold: 0.75, debounce: 1000, api_url: 'http://my.custom.backend/api'}
 *   );
 */
export default function hookInfiniteScroll(builder, opt) {
	if (opt === undefined) opt = {};
	// we could do it with less lines using || but it checks for truthy (e.g. would discard debounce 0)
	// explicitly check for undefined to avoid this, it's done only once at the start anyway
	let container_id = opt.container_id != undefined ? opt.container_id : "#container";
	let threshold = opt.threshold != undefined ? opt.threshold : 0.9;
	let debounce = opt.debounce != undefined ? opt.debounce : 250;
	let api_url = opt.api_url != undefined ? opt.api_url : "/api";
	let prefetch = opt.prefetch != undefined ? opt.prefetch : true;
	let offset_arg = opt.offset_arg != undefined ? opt.offset_arg : "offset";
	let free_index = opt.free_index != undefined ? opt.free_index : true;

	let container = document.getElementById(container_id);
	let last_activation = Date.now();
	let offset = 0;

	function scrollDepth() {
		let html_tag = document.body.parentElement;
		return html_tag.scrollTop / (html_tag.scrollHeight - html_tag.clientHeight)
	}

	let callback = async (ev) => {
		if (ev !== true && scrollDepth() < threshold) return; // not scrolled enough
		if (ev !== true && Date.now() - last_activation < debounce) return; // triggering too fast
		last_activation = Date.now();
		let response = await fetch(`${api_url}?${offset_arg}=${offset}`);
		let replies = await response.json();
		if (replies.length == 0) {
			// reached end, unregister self
			return document.removeEventListener("scroll", callback);
		}
		offset += free_index ? replies.length : 1; // track how deep we went
		for (let repl of replies) {
			container.innerHTML += builder(repl);
		}
	};

	document.addEventListener("scroll", callback);
	if (prefetch) callback(true); // invoke once immediately
}
