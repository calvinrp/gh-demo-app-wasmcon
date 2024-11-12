#[allow(warnings)]
mod bindings;

use bindings::wasi::http::types::{Method, Scheme};
use serde::{Deserialize, Serialize};

use crate::bindings::{
    exports::wasi::http::incoming_handler::{Guest, IncomingRequest, ResponseOutparam},
    wasi::http::types::{Fields, OutgoingBody, OutgoingResponse},
    wasi::io::streams::StreamError,
};
use bindings::wasi::http::outgoing_handler::{handle, OutgoingRequest};

const TOKEN: &[u8] = b"Bearer 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";

const HOME: &[u8] = b"
<html>
<head>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
  tailwind.config = {
    theme: {
      extend: {},
      fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
      },
  },
  }
</script>
<title>GitHub Issue Manager</title></head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script type=\"module\">
      async function getIssues() {
        let res = await fetch(\"/gh\");
        return res;
      }
      let issuesRes = await getIssues();
      let issues = await issuesRes.json();

      let ul = document.createElement(\"ul\");
      ul.classList.add('mt-5', 'list-disc', 'pl-5');
      for (const issue of issues) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        a.classList.add(\"underline\", \"decoration-sky-500\", \"decoration-solid\", \"decoration-2\", 'text-slate-200', 'hover:text-white');
        li.appendChild(a);
        const linkText = document.createTextNode(issue.title);
        a.appendChild(linkText);
        a.href = `/issue?owner=calvinrp&repo=WasmCon-issues-demo&number=${issue.number}`;
        ul.appendChild(li);
      }
      document.body.appendChild(ul);

    </script>
    <h1 class=\"text-3xl text-slate-100 font-black mb-10\">GitHub Issue Manager</h1>
    <a href=\"/create\" class=\"inline-block py-2 px-3 bg-slate-800 rounded-lg text-slate-200 hover:text-white hover:bg-slate-700\">Create an Issue</a>
    <h4 class=\"text-lg text-slate-300 font-semibold mb-5 mt-10\">Open Issues:</h4>
    </body>
    </html>
    ";

const ISSUE: &[u8] = b"
<html>
<head>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
  tailwind.config = {
    theme: {
      extend: {},
      fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
      },
  },
  }
</script>
<title></title>
    </head>
    <body class=\"bg-slate-900 text-white p-5\">
<h1 class=\"text-3xl text-slate-100 font-black mb-10\" id=\"issue-title\"></h1>
<div class=\"text-slate-300 bg-slate-800 rounded p-3 inline-block\" id=\"issue-body\">
</div>
<div class=\"mt-10\">
<a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</div>
<script type=\"module\">
  const urlParams = new URLSearchParams(window.location.search);
  async function getIssue() {
    let res = await fetch(`/gh/issue?${urlParams.toString()}`);
    return res;
  }
  const issuesRes = await getIssue();
  const issue = await issuesRes.json();
  
  const h1 = document.getElementById('issue-title');
  h1.textContent = issue.title;
  document.title = issue.title;

  const issueBody = document.getElementById('issue-body');
  issueBody.setHTMLUnsafe(issue.body);
</script>
</body>
</html>";

const CREATE: &[u8] = b"
<html>
<head>
<title>Create new issue</title>
<script src=\"https://cdn.tailwindcss.com\">
</script>
<script>
  tailwind.config = {
    theme: {
      extend: {},
      fontFamily: {
        sans: [\"Inter var\", \"sans-serif\"],
        mono: [\"Roboto Mono\", \"monospace\"],
      },
  },
  }
</script>
</head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script type=\"module\">
      const urlParams = new URLSearchParams(window.location.search);
      const form = document.getElementById(\"form\");
      async function submitHandler(e) {
        const data = new FormData(e.target);
        e.preventDefault()
        const body = JSON.stringify({
          title: data.get('title'),
          body: data.get('body'),
        });
        let res = await fetch(`/gh/create`, {
          method: \"POST\",
          headers: { 'Content-Type': 'application/json' },
          body,
        });
        let resBody = await res.json();

        window.location.pathname = '/';
      }
      form.addEventListener(\"submit\", submitHandler);
    </script>
<h1 class=\"text-3xl text-slate-100 font-black mb-10\">Create new issue</h1>
<div class=\"mb-10\">
<form id=\"form\">
  <label for=\"title\">Issue Title</label>
  <div class=\"mb-3 mt-1\">
      <input type=\"text\" id=\"title\" name=\"title\" class=\"bg-slate-700 text-slate-100 py-2 px-3 rounded\">
  </div>
  <label for=\"body\">Issue Body</label>
  <div class=\"mt-1\">
  <textarea id=\"issue-body\" name=\"body\", rows=\"4\" cols=\"50\" class=\"bg-slate-700 text-slate-100 py-2 px-3 rounded\">Write your issue here</textarea>
  </div>
  <div class=\"mt-5\">
   <input type=\"submit\" value=\"Submit\" class=\"cursor-pointer inline-block py-2 px-3 bg-slate-800 rounded-lg text-slate-200 hover:text-white hover:bg-slate-700\" />
  </div>
</form>
</div>
<a href=\"/\" class=\"underline decoration-sky-500 decoration-solid decoration-2 text-slate-200 hover:text-white\">Back to issue list</a>
</body>
</html>";

struct Component;

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    title: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Params {
    owner: String,
    repo: String,
    number: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CreateRequest {
    title: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ReqBody {
    title: String,
    body: String,
}

impl Guest for Component {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let path = request.path_with_query().unwrap();
        if path.starts_with("/create") {
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(CREATE)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        } else if path.starts_with("/issue") {
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(ISSUE)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        } else if &path == "/gh" {
            let fields = Fields::new();
            fields
                .set(
                    &String::from("Accept"),
                    &["application/vnd.github+json".as_bytes().to_vec()],
                )
                .unwrap();
            fields
                .set(
                    &String::from("Authorization"),
                    &[TOKEN
                        .to_vec()],
                )
                .unwrap();
            fields
                .set(
                    &String::from("X-GitHub-Api-Version"),
                    &["2022-11-28".as_bytes().to_vec()],
                )
                .unwrap();
            let req = OutgoingRequest::new(fields);
            req.set_method(&Method::Get).unwrap();
            req.set_scheme(Some(&Scheme::Https)).unwrap();
            req.set_path_with_query(Some(&format!("/repos/calvinrp/WasmCon-issues-demo/issues")))
                .unwrap();
            req.set_authority(Some("api.github.com")).unwrap();
            let future_res = handle(req, None).expect("future response");
            future_res.subscribe().block();

            let res = future_res
                .get()
                .unwrap()
                .map_err(|err| anyhow::anyhow!("outgoing response error code: {err:?}"))
                .unwrap()
                .unwrap();
            let body = res.consume().unwrap();
            let stream = body.stream().unwrap();
            let bytes = &stream.blocking_read(100000000000).unwrap();
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(bytes)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        } else if path.starts_with("/gh/issue") {
            let fields = Fields::new();
            fields
                .set(
                    &String::from("Accept"),
                    &["application/vnd.github+json".as_bytes().to_vec()],
                )
                .unwrap();
            fields
                .set(
                    &String::from("Authorization"),
                    &[TOKEN
                        .to_vec()],
                )
                .unwrap();
            fields
                .set(
                    &String::from("X-GitHub-Api-Version"),
                    &["2022-11-28".as_bytes().to_vec()],
                )
                .unwrap();
            let req = OutgoingRequest::new(fields);
            req.set_method(&Method::Get).unwrap();
            req.set_scheme(Some(&Scheme::Https)).unwrap();
            let pieces = path.split("?");
            let param_string = pieces.last().unwrap();
            let params: Params = serde_qs::from_str(param_string).unwrap();

            req.set_path_with_query(Some(&format!(
                "/repos/{}/{}/issues/{}",
                params.owner, params.repo, params.number
            )))
            .unwrap();
            req.set_authority(Some("api.github.com")).unwrap();
            let future_res =
                handle(req, None).map_err(|err| anyhow::anyhow!("outgoing error code: {err}"));
            let future_res = future_res.unwrap();
            let future_res_pollable = future_res.subscribe();
            future_res_pollable.block();

            let res = future_res
                .get()
                .unwrap()
                .map_err(|err| anyhow::anyhow!("outgoing response error code: {err:?}"))
                .unwrap()
                .unwrap();
            let body = res.consume().unwrap();
            let stream = body.stream().unwrap();
            let bytes = &stream.blocking_read(100000000000).unwrap();
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(bytes)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        } else if path.starts_with("/gh/create") {
            let wasm_body = request.consume().unwrap();
            let wasm_stream = wasm_body.stream().unwrap();
            let mut bytes = Vec::new();
            loop {
                match wasm_stream.blocking_read(10000000000000000000) {
                    Ok(mut b) => {
                        bytes.append(&mut b);
                    }
                    Err(StreamError::Closed) => {
                        break;
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
            }
            let issue = serde_json::from_slice::<CreateRequest>(&bytes).expect("valid JSON");
            let fields = Fields::new();
            fields
                .set(
                    &String::from("Accept"),
                    &["application/vnd.github+json".as_bytes().to_vec()],
                )
                .unwrap();
            fields
                .set(
                    &String::from("Authorization"),
                    &[TOKEN
                        .to_vec()],
                )
                .unwrap();
            fields
                .set(&String::from("User-Agent"), &["Foo".as_bytes().to_vec()])
                .unwrap();
            fields
                .set(
                    &String::from("X-GitHub-Api-Version"),
                    &["2022-11-28".as_bytes().to_vec()],
                )
                .unwrap();
            let req = OutgoingRequest::new(fields);
            req.set_method(&Method::Post).unwrap();
            req.set_scheme(Some(&Scheme::Https)).unwrap();
            req.set_path_with_query(Some(&format!("/repos/calvinrp/WasmCon-issues-demo/issues",)))
                .unwrap();
            req.set_authority(Some("api.github.com")).unwrap();
            let body = req.body().unwrap();
            let stream = body.write().unwrap();
            let md_opt = markdown::Options::gfm();
            let issue_body = markdown::to_html_with_options(&issue.body, &md_opt).unwrap();
            let to_write = ReqBody {
                title: issue.title,
                body: issue_body,
            };
            stream
                .blocking_write_and_flush(serde_json::to_string(&to_write).unwrap().as_bytes())
                .unwrap();
            let future_res =
                handle(req, None).map_err(|err| anyhow::anyhow!("outgoing error code: {err}"));
            let future_res = future_res.unwrap();
            let future_res_pollable = future_res.subscribe();
            drop(stream);
            OutgoingBody::finish(body, None).unwrap();
            future_res_pollable.block();

            let res = future_res.get();
            let res = match res.unwrap().unwrap() {
                Ok(res) => res,
                Err(err) => {
                    dbg!(err);
                    panic!("error http outgoing");
                }
            };
            let body = res.consume().unwrap();
            let stream = body.stream().unwrap();
            let bytes = &stream.blocking_read(100000000000).unwrap();
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(bytes)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        } else {
            let hdrs = Fields::new();
            let resp = OutgoingResponse::new(hdrs);
            let body = resp.body().expect("outgoing response");

            ResponseOutparam::set(response_out, Ok(resp));
            let out = body.write().expect("outgoing stream");
            out.blocking_write_and_flush(HOME)
                .expect("writing response");

            drop(out);
            OutgoingBody::finish(body, None).unwrap();
        }
    }
}

bindings::export!(Component with_types_in bindings);
