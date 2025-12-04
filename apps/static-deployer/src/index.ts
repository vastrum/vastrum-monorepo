import {
  register_static_route,
} from "vastrum-lib";

export function deploy() {

}
export function add_page(url: string, html_content: string) {
  register_static_route(url, html_content);
}
