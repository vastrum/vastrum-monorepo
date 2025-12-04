import {
  u64,
  block_time,
  register_static_route,
  Repository,
} from "vastrum-lib";
import { Category, Post, Post_Reply } from "./types";
import { post_view } from "./templates/post";
import { post_catalogue_view } from "./templates/post_catalogue";
import { post_catalogue_url, post_url } from "./utilities";

const post_repo = new Repository(Post, 0);
const post_replies_repo = new Repository(Post_Reply, 1);
const categories_repo = new Repository(Category, 2);

function render_post_catalogue() {
  let posts = post_repo.query().limit(100).orderBy("last_activity_time", "desc").get();
  register_static_route(post_catalogue_url(), post_catalogue_view(posts));
}

function render_post(post: Post) {
  let repliesWithSameParentPost = post_replies_repo
    .query()
    .where("parent_post_id", "==", post.id)
    .orderBy("time", "asc")
    .limit(100)
    .get();

  register_static_route(post_url(post), post_view(post, repliesWithSameParentPost));
}



export function deploy() {
  post_repo.createTable();
  post_replies_repo.createTable();
  categories_repo.createTable();
  categories_repo.insert({ id: 0, posts_length: 0 })
  render_post_catalogue();
}

export function create_post(post_content: string, post_title: string) {
  let current_time = block_time();

  const post: Post = {
    id: monotonic_post_id(),
    time: current_time,
    content: post_content,
    title: post_title,
    replies_length: 0,
    last_activity_time: current_time,
  };
  post_repo.insert(post);

  render_post(post);
  render_post_catalogue();
  save_all_posts();
}

export function create_post_reply(content: string, parent_post_id: u64) {
  let current_time = block_time();

  let parent_post = post_repo
    .query()
    .where("id", "==", parent_post_id)
    .get()[0];
  parent_post.replies_length += 1;
  parent_post.last_activity_time = current_time;
  post_repo.update(parent_post.id, parent_post);


  let post_reply: Post_Reply = {
    id: parent_post.replies_length,
    time: current_time,
    content: content,
    parent_post_id: parent_post_id,
  };
  post_replies_repo.insert(post_reply);

  render_post(parent_post);
  render_post_catalogue();
  save_all_replies();
}


function save_all_posts() {
  let all_posts = post_repo
    .query()
    .limit(10000)
    .get();
  register_static_route("postsjson", JSON.stringify(all_posts));
}
function save_all_replies() {
  let all_post_replies = post_replies_repo.query().limit(10000).get();
  register_static_route("repliesjson", JSON.stringify(all_post_replies));
}

function monotonic_post_id(): number {
  let category = categories_repo.query().where("id", "==", 0).get()[0];
  category.posts_length += 1;
  categories_repo.update(category.id, category);
  return category.posts_length;
}