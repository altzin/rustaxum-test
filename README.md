README

currently working on adding nats and bookmark worker written in go. reason for go is it comes with exactly the tools i need for the workers. 

Bookmark worker
is a go project that should be scalable as it just pulls from nats/jetstream. handling failures with a dlq any data it needs is accessed through the bff axum/rust api. axum creates the job by pushing if thats the right word to the jetsream and writes some data about the current status to postgres. pending/processing/completed/failed.

The Fast Path (NATS): Axum saves to DB as pending and publishes to JetStream. The Go worker receives the event, instantly calls Axum (PATCH /api/bookmarks/:id/status with status: processing), generates the PDF, uploads it, and calls Axum again with status: completed and the storage URL.

trransaction level advisory locks in axum from postgres handles the sweep stuff to make sure failed gets reprocessed or not.

Rows where status == 'pending' older than 5 minutes (NATS dropped the message).

Rows where status == 'processing' older than 15 minutes (the Go worker crashed mid-generation).
The sweep simply republishes these IDs to JetStream and updates the timestamp.

If we sha the urls to create names for the object we save as a bookmark we can check if it already exists and thus just write again if we fail a write from gen->db.

Tracing a request is important now. we want to check that the latency of a write to the db is not slow because of the long run time in generators. so we pass a trace context. we also need metrics from postgres,jetstream via vector or prom, toggle the stat thing for postgres to get the juicy bits. or span links.A

generators catch poison pill after 3 retries and send to DLQ.

we are not saving pdfs. we generate md from urls and store images separatley and link them together as we need them. 

we also wait for body, i dont care about css or ads, so as soon as body is fetched start the process.

scrub metadata before we generate a md file. handle the images somehow? saved to garageHQ or whatever. if garage handles s3 one disk shared over the cluster

we will need an extension. build one and publish it to moz firefox, dont hardode a url set one as a config in the addon. or swap to brave which might be decent anyway.

dont limit uploads, just handle them. in the sweep which runs once a minute. maybe limit it so we have time to drain to jetstream and swap to processing. it depends on how we write sweeps later. drain until your empty once a minute. being stuck in processing should be fine, its observable and could be fixed in the future. if we added obs to nats/js
