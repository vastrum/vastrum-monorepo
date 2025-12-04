import { Column, DataType } from "vastrum-lib";


export class Post {
  @Column(DataType.Uint64)
  id!: number;

  @Column(DataType.Uint64)
  time!: number;

  @Column(DataType.String)
  content!: string;

  @Column(DataType.String)
  title!: string;

  @Column(DataType.Uint64)
  replies_length!: number;

  @Column(DataType.Uint64)
  last_activity_time!: number;

  constructor(
    id: number = 0,
    time: number = 0,
    content: string = "",
    title: string = "",
    replies_length: number = 0,
    last_activity_time: number = 0,
  ) {
    this.id = id;
    this.time = time;
    this.content = content;
    this.title = title;
    this.replies_length = replies_length;
    this.last_activity_time = last_activity_time;
  }
}

export class Post_Reply {
  @Column(DataType.Uint64)
  id!: number;

  @Column(DataType.Uint64)
  time!: number;

  @Column(DataType.String)
  content!: string;

  @Column(DataType.Uint64)
  parent_post_id!: number;

  constructor(
    id: number = 0,
    time: number = 0,
    content: string = "",
    parent_post_id: number = 0
  ) {
    this.id = id;
    this.time = time;
    this.content = content;
    this.parent_post_id = parent_post_id;
  }
}



export class Category {
  @Column(DataType.Uint64)
  id!: number;

  @Column(DataType.Uint64)
  posts_length!: number;

  constructor(
    id: number = 0,
    posts_length: number = 0,
  ) {
    this.id = id;
    this.posts_length = posts_length;
  }
}
