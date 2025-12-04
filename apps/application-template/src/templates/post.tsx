import { renderToString } from "preact-render-to-string";
import { h } from "preact";
import { Post, Post_Reply } from "../types";
import { ThumbsUp } from "lucide-preact";
import { formatTimestamp, href_post_catalogue_url, post_catalogue_url, site_name } from "../utilities";



export function post_view(post: Post, replies: Post_Reply[],
): string {
  return renderToString(
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b border-gray-200 sticky top-0 z-10">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <h1 className="text-2xl font-bold text-gray-900">{site_name()}</h1>

            <button
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 cursor-pointer"
              data-href={href_post_catalogue_url()}>
              Go to post catalogue
            </button>
          </div>
        </div>
      </header >

      <div className="max-w-4xl mx-auto px-4 py-6">
        <div className="bg-white rounded-lg shadow">
          <div className="p-6 border-b border-gray-200">
            <div className="mb-4">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">{post.title}</h2>
              <div className="flex items-center gap-4 text-sm text-gray-500 mb-4">
                <span className="font-medium text-gray-700">{formatTimestamp(post.time, 'YYYY-MM-DD HH:mm:ss')}</span>
              </div>
              <p className="text-gray-700">{post.content}</p>
            </div>
          </div>

          {replies.map((reply, idx) => (
            <div key={reply.id} className={`p-6 ${idx !== replies.length - 1 ? 'border-b border-gray-200' : ''}`}>
              <div className="flex gap-4">
                <div className={`w-12 h-12 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold flex-shrink-0`}>
                  A
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="font-semibold text-gray-900">ANON</span>
                    <span className="text-sm text-gray-500">{formatTimestamp(reply.time, 'YYYY-MM-DD HH:mm:ss')}</span>
                  </div>
                  <p className="text-gray-700 mb-3">{reply.content}</p>
                </div>
              </div>
            </div>
          ))}

          <div className="p-6 bg-gray-50">
            <textarea
              id="content"
              className="w-full resize-none p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              // @ts-ignore
              rows="4"
            />
            <div className="mt-3 flex justify-end">

              <button
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 cursor-pointer"
                data-signature="create_post_reply"
                data-args={"content,parent_post_id=" + post.id}
              >
                Post Reply
              </button>
            </div>
          </div>

        </div>
      </div>
    </div >
  );
}

