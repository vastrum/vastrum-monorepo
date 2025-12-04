import { describe, it, expect, vi, beforeEach, test, assert } from "vitest";
import { Repository } from "./dblib";
import { Column, DataType } from "./typeReflector";
import {
  db_create_table,
  db_insert_entry,
  db_query,
  db_update_entry,
  type DatabaseInsertEntry,
  type DatabaseQuery,
  type DatabaseUpdateEntry,
} from "./index";

//First id value becomes primary key
export class Post {
  @Column(DataType.Uint64)
  id!: number;

  @Column(DataType.String)
  title!: string;

  @Column(DataType.String)
  content!: string;

  @Column(DataType.Uint64)
  authorId!: number;

  @Column(DataType.Bool)
  islast!: boolean;

  constructor(
    id: number = 0,
    title: string = "",
    content: string = "",
    authorId: number = 0,
    islast: boolean = false
  ) {
    this.id = id;
    this.title = title;
    this.content = content;
    this.authorId = authorId;
    this.islast = islast;
  }
}

// Mock the entire module
vi.mock("./index");

describe("dblib", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Query builder", () => {
    it("Should construct correct queries and parse return values", async () => {
      const mockDbQuery = vi
        .fn()
        .mockReturnValue([["0", "titlevalue", "contentvalue", "123", "true"]]);

      vi.mocked(db_query).mockImplementation(mockDbQuery);

      // Act
      const repo = new Repository(Post, 0);
      repo.createTable();

      const res = repo
        .query()
        .where("authorId", "==", "3")
        .where(repo.columns.authorId, "==", "100")
        .orderBy("title", "desc")
        .limit(5)
        .offset(3)
        .get();

      const expected_query: DatabaseQuery = {
        table_id: 0,
        number_of_fields: 5,
        sorting_operations: [{ field_id: 2, descending: true }],
        where_operations: [
          { field_id: 4, value: "3" },
          { field_id: 4, value: "100" },
        ],
        limit: 5,
        offset: 3,
      };
      // query building works
      expect(mockDbQuery).toHaveBeenCalledWith(expected_query);

      expect(res[0].id).toBe(0);
      expect(res[0].title).toBe("titlevalue");
      expect(res[0].content).toBe("contentvalue");
      expect(res[0].authorId).toBe(123);
      expect(res[0].islast).toBe(true);
    });

    it("Should insert entries with correct format", async () => {
      const mockDbInsert = vi.fn().mockReturnValue(null);

      vi.mocked(db_insert_entry).mockImplementation(mockDbInsert);

      const repo = new Repository(Post, 0);
      repo.createTable();

      repo.insert(new Post(0, "titlevalue", "contentvalue", 123, true));
      const expected_insert: DatabaseInsertEntry = {
        table_id: 0,
        data: ["0", "titlevalue", "contentvalue", "123", "true"],
      };
      expect(mockDbInsert).toHaveBeenCalledWith(expected_insert);
    });

    it("Should update entries with correct format", async () => {
      const mockDbUpdate = vi.fn().mockReturnValue(null);

      vi.mocked(db_update_entry).mockImplementation(mockDbUpdate);

      const repo = new Repository(Post, 0);
      repo.createTable();

      repo.update(5, new Post(0, "titlevalue", "contentvalue", 123, true));

      const expected_insert: DatabaseUpdateEntry = {
        table_id: 0,
        select_on_primary_key_value: "5",
        data: ["titlevalue", "contentvalue", "123", "true"],
      };
      expect(mockDbUpdate).toHaveBeenCalledWith(expected_insert);
    });
  });
});
