#[allow(warnings)]
mod bindings;

use bindings::wasi::http::types::{Method, Scheme};
use serde::{Deserialize, Serialize};

use crate::bindings::{
    exports::wasi::http::incoming_handler::{Guest, IncomingRequest, ResponseOutparam},
    wasi::http::types::{Fields, OutgoingBody, OutgoingResponse},
};
use bindings::wasi::http::outgoing_handler::{handle, OutgoingRequest};
const HOME: &[u8] = b"
<html>
<head>
    <title>Example page</title>
    </head>
    <body>
    <script type=\"module\">
      async function getIssues() {
        let res = await fetch(\"/_/gh\");
        return res;
      }
      let issuesRes = await getIssues();
      let issues = await issuesRes.json();

      let ol = document.createElement(\"ol\");
      for (const issue of issues) {
        let li = document.createElement(\"li\");
        let a = document.createElement(\"a\");
        li.appendChild(a);
        const linkText = document.createTextNode(issue.title);
        a.appendChild(linkText);
        a.href = `/issue?owner=${issue.repository.owner.login}&repo=${issue.repository.name}&number=${issue.number}`;
        ol.appendChild(li);
      }
      document.body.appendChild(ol);

    </script>
    <h1>Home page</h1>
    <a href=\"/create\">Create an Issue</a>
    </div>
    </body>
    </html>
    ";

const ISSUE: &[u8] = b"
<html>
<head>
    <title>Example page</title>
    </head>
    <body>
    <script type=\"module\">
      const urlParams = new URLSearchParams(window.location.search);
      async function getIssue() {
        let res = await fetch(`/_/gh/issue?${urlParams.toString()}`);
        return res;
      }
      let issuesRes = await getIssue();
      let issue = await issuesRes.json();
      console.log({issue})
      let h1 = document.createElement(\"h1\");
      h1.textContent = issue.title;
      document.body.appendChild(h1);
      let body = document.createElement(\"div\");
      body.textContent = issue.body
      document.body.appendChild(body);
    </script>
<div>
</div>
</body>
</html>";

const CREATE: &[u8] = b"
<html>
<head>
    <title>Create an issue</title>
    </head>
    <body>
    <script type=\"module\">
      const urlParams = new URLSearchParams(window.location.search);
      const form = document.getElementById(\"form\");
      async function submitHandler(e) {
        console.log({e})
        const data = new FormData(e.target);
        e.preventDefault()
        const [[_title, title], [_repo, repo], [_owner, owner], [_body, body]] = [...data.entries()]
        console.log([...data.entries()])
        let res = await fetch(`/_/gh/create?}`, {
          method: \"POST\",
          body: JSON.stringify({
            title,
            body,
            repo,
            owner
          })
        });
        return res;
      }
      form.addEventListener(\"submit\", submitHandler);
    </script>
<div>
<form id=\"form\">
  <label for=\"title\">Issue Title</label>
  <input type=\"text\" id=\"title\" name=\"title\"><br><br>
  <label for=\"repo\">Repo Name</label>
  <input type=\"text\" id=\"repo\" name=\"repo\"><br><br>
  <label for=\"owner\">Repo Owner</label>
  <input type=\"text\" id=\"owner\" name=\"owner\"><br><br>
  <label for=\"body\">Issue Body:</label>
  <textarea id=\"issue-body\" name=\"body\", rows=\"4\" cols=\"50\">Write your issue here</textarea>
  <input  type=\"submit\" value=\"Submit\">
</form>
</div>
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
    owner: String,
    repo: String,
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
        dbg!(&path);
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
            req.set_path_with_query(Some("/issues")).unwrap();
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
            dbg!(&params);

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
            let mut more_bytes = true;
            while more_bytes {
                if let Ok(new_bytes) = wasm_stream.blocking_read(10000000000000000000) {
                    bytes.extend(new_bytes);
                } else {
                    more_bytes = false;
                }
            }
            let issue = serde_json::from_slice::<CreateRequest>(&bytes).unwrap();
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
            req.set_path_with_query(Some(&format!(
                "/repos/{}/{}/issues",
                issue.owner, issue.repo
            )))
            .unwrap();
            req.set_authority(Some("api.github.com")).unwrap();
            let body = req.body().unwrap();
            let stream = body.write().unwrap();
            let to_write = ReqBody {
                title: issue.title,
                body: issue.body,
            };
            stream
                .blocking_write_and_flush(serde_json::to_string(&to_write).unwrap().as_bytes())
                .unwrap();
            let future_res =
                handle(req, None).map_err(|err| anyhow::anyhow!("outgoing error code: {err}"));
            let future_res = future_res.unwrap();
            let future_res_pollable = future_res.subscribe();
            future_res_pollable.block();

            let res = future_res.get();
            // .map_err(|err| anyhow::anyhow!("outgoing response error code: {err:?}"));
            dbg!(&res);
            let res = res.unwrap().unwrap().unwrap();
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
