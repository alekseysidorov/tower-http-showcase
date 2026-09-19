# Architecture decisions

## Keep the domain API separate from Tower transport

`HelloService` represents the typed remote operation. `HelloClient<S>` adapts
that operation to a generic Tower service and exposes `S::Error` as its
transport error. HTTP status failures and response decoding failures are
separate from service failures. Tower `Service` remains the HTTP execution
abstraction; the showcase does not add a second generic `execute()` trait.

## Normalize reqwest bodies at one boundary

The transport adapter accepts `Full<Bytes>` so requests are cloneable and can
be replayed by Tower retry middleware. It converts that body to `reqwest::Body`
immediately before `HttpClientLayer`, then maps response bodies to
`BoxBody<Bytes, BoxError>`. The concrete reqwest body types stop at this
adapter.

## Keep specialized middleware specialized

Each backend receives a `RewriteUriLayer` with its own base URI. Logging and
header mutation remain separate middleware. Retry is applied to each node
service; `PeakEwma` and `Balance` then select between those node stacks, with
buffering and concurrency limiting composed around the balanced service.
Type erasure is delayed until the composed client must be cloneable and
homogeneous for concurrent calls.

## Delegate readiness to the client extension

`HelloClient` sends requests using `tower_http_client::ServiceExt::send`. Its
implementation reaches `ServiceExt::execute`, which awaits readiness and calls
the same service instance. The domain adapter therefore does not duplicate
Tower readiness polling or clone a service between readiness and `call`.
