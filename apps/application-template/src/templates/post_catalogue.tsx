import { Post } from "../types";

import { renderToString } from "preact-render-to-string";
import { h } from "preact";
import { formatTimestamp, href_post_url, post_url, site_name } from "../utilities";
import { MessageSquare, Users, Eye, ThumbsUp, Pin, Lock } from 'lucide-preact';



export function post_catalogue_view(posts: Post[]): string {
  return renderToString(
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 sticky top-0 z-10">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-gray-900">{site_name()}</h1>
          </div>
        </div>
      </header>

      <div className="max-w-6xl mx-auto px-4 py-6">
        <div className="flex gap-4 mb-6 overflow-x-auto">
          {<button
            className={`px-4 py-2 rounded-lg whitespace-nowrap transition-colors bg-blue-600 text-white`}
          >
            {"All Topics"}
          </button>
          }
        </div>

        <div className="bg-white rounded-lg shadow overflow-hidden ">
          {posts.map((post, idx) => (
            <button
              className={`p-4 hover:bg-gray-50 cursor-pointer transition-colors w-full border-b border-gray-200`}
              data-href={href_post_url(post)}
            >
              <div className="flex items-start gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="text-lg font-semibold text-gray-900 hover:text-blue-600">
                      {post.title}
                    </h3>
                  </div>
                  <div className="flex items-center gap-4 text-sm text-gray-500">
                    <span className="font-medium text-gray-700">ANON</span>
                    <span>•</span>
                    <span className="font-medium text-gray-700">{formatTimestamp(post.time, 'YYYY-MM-DD HH:mm:ss')}</span>
                  </div>
                </div>
                <div className="flex gap-4 text-sm text-gray-500">
                  <div className="flex items-center gap-1">
                    <MessageSquare className="w-4 h-4" />
                    <span>{post.replies_length}</span>
                  </div>
                </div>
              </div>
            </button>

          ))}
        </div>


        <div className="mt-6 bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Create New Post</h3>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Title</label>
              <input
                id="post_title"
                type="text"
                className="w-full p-3 border border-fray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Content</label>
              <textarea
                id="post_content"
                className="w-full resize-none p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                // @ts-ignore
                rows="6"
              />
            </div>
            <div className="flex justify-end">
              <button
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 cursor-pointer" data-signature="create_post"
                data-args="post_content,post_title"
              >
                Create Post
              </button>
            </div>
          </div>
        </div>



      </div>
    </div >
  );
}



