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
<style>
  body a {
    color: #7dd3fc;
  }
</style>
<title>Issue Manager</title></head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script type=\"module\">
      async function getIssues() {
        let res = await fetch(\"/gh\");
        return res;
      }
      let issuesRes = await getIssues();
      let issues = await issuesRes.json();
      console.log()

      let ol = document.createElement(\"ol\");
      for (const issue of issues) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        li.appendChild(a);
        const linkText = document.createTextNode(issue.title);
        a.appendChild(linkText);
        a.href = `/issue?owner=JAFLabs&repo=issues&number=${issue.number}`;
        ol.appendChild(li);
      }
      document.body.appendChild(ol);

    </script>
    <h1>Issue Manager</h1>
    <a href=\"/create\">Create an Issue</a>
    </div>
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
<style>
  body a {
    color: #7dd3fc;
  }
</style>
    <title>Issue</title>
    </head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script src=\"https://cdn.jsdelivr.net/npm/marked/marked.min.js\"></script>

    <script type=\"module\">
      const urlParams = new URLSearchParams(window.location.search);
      async function getIssue() {
        let res = await fetch(`/gh/issue?${urlParams.toString()}`);
        return res;
      }
      let issuesRes = await getIssue();
      let issue = await issuesRes.json();
      let h1 = document.createElement(\"h1\");
      h1.textContent = issue.title;
      document.body.appendChild(h1);
      let issueBody = document.createElement(\"div\");
      // issueBody.setAttribute(\"markdown\", \"1\");
      issueBody.innerHTML = issue.body;
      // issueBody.appendChild(issueText);

      console.log(marked.parse(issue.body))

      document.body.appendChild(issueBody);
    </script>
<div>
</div>
</body>
</html>";

const CREATE: &[u8] = b"
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
<style>
  body a {
    color: #7dd3fc;
  }

  body form {
    border-radius: .5rem;
    background-color: #1e293b;
  }
  body input {
    border-radius: .5rem;
    background-color: #334155;
  }
  body textarea {
    border-radius: .5rem;
    background-color: #334155;
    box-sizing: border-box;
  }
</style>
    <title>Create an issue</title>
    </head>
    <body class=\"bg-slate-900 text-white p-5\">
    <script type=\"module\">
      const urlParams = new URLSearchParams(window.location.search);
      const form = document.getElementById(\"form\");
      async function submitHandler(e) {
        console.log(\"SUBMITTING\");
        const data = new FormData(e.target);
        e.preventDefault()
        const body = JSON.stringify({
          title: data.get('title'),
          body: data.get('body'),
        });
        console.log({body})
        let res = await fetch(`/gh/create`, {
          method: \"POST\",
          headers: { 'Content-Type': 'application/json' },
          body,
        });
        let resBody = await res.json();
        console.log({resBody})
        let success = document.createElement(\"a\");
        const linkText = document.createTextNode(\"Successfully created issue\");
        success.appendChild(linkText);
        success.href = resBody.html_url;
        document.body.appendChild(success);
        console.log({resBody})

        return res;
      }
      form.addEventListener(\"submit\", submitHandler);
    </script>
<div>
<form id=\"form\">
  <label for=\"title\">Issue Title</label>
  <div>
  <input type=\"text\" id=\"title\" name=\"title\"><br><br>
  </div>
  <label for=\"body\">Issue Body</label>
  <div>
  <textarea id=\"issue-body\" name=\"body\", rows=\"4\" cols=\"50\">Write your issue here</textarea>
  </div>
  <div>
  <input  type=\"submit\" value=\"Submit\">
  </div>
</form>
</div>
<a href=\"/\">Back to issue list</a>
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
                    &["Bearer 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"
                        .as_bytes()
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
            req.set_path_with_query(Some(&format!("/repos/JAFLabs/issues/issues")))
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
                    &["Bearer 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"
                        .as_bytes()
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
            dbg!(&issue);
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
                    &["Bearer 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"
                        .as_bytes()
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
            req.set_path_with_query(Some(&format!("/repos/JAFLabs/issues/issues",)))
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
            dbg!(&to_write);
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
