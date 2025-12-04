import { Post } from "./types";

export function post_url(post: Post) {
  return `post/${post.id}`;
}

export function post_catalogue_url() {
  return 'postcatalogue';
}

export function site_name() {
  return 'Windhall';
}

function domain_name() {
  return "zkpunks";
}
export function href_post_url(post: Post) {
  return domain_name() + "/" + post_url(post);
}
export function href_post_catalogue_url() {
  return domain_name() + "/" + post_catalogue_url();
}

export function formatTimestamp(timestamp: number, format: string): string {
  const date = new Date(timestamp * 1000);

  const pad = (n: number) => n.toString().padStart(2, '0');

  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());
  const hours12 = pad(date.getHours() % 12 || 12);
  const ampm = date.getHours() >= 12 ? 'PM' : 'AM';

  return format
    .replace('YYYY', year.toString())
    .replace('MM', month)
    .replace('DD', day)
    .replace('HH', hours)
    .replace('hh', hours12)
    .replace('mm', minutes)
    .replace('ss', seconds)
    .replace('A', ampm);
}