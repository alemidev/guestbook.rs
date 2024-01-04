# guestbook.rs
a super simple guestbook to deploy on your site: just grab the binary and run it, or configure it to your liking

try it out [here](https://guestbook.alemi.dev/) with a test instance _(db cleared every hour, default settings)_, or check out a more integrated deployment [here, at the bottom](https://alemi.dev/)

# why
i love small web and i want to give a way to whoever visits my site to leave a mark. cooking something to do that is very easy, but after connecting my third bare form i decided i needed something more convenient.

# deploy
## all-in-one
`guestbook.rs` can be easily deployed without anything else: just grab (or compile) the default binary and run it:
```sh
guestbook serve
```

it will create a sqlite database (`guestbook.db`) in current directory and start serving its frontend on http://localhost:37812/ and its backend on http://localhost:37812/api

all comments will be public and will only be logged on console

## simple configuration
it's possible to configure frontend style, extra notifiers and page overrides using a `.toml` config file

if no config file is provided, the implicit default will be used (view it with `guestbook config` command)

a config file must be specified with the `-c / --config` CLI argument

### overrides
by default each user is given full control on each page (except for the `date` field). this can be changed with overrides.

as an example, you may want to review posts before showing them publicly.

add following section to your `config.toml`:
```toml
[overrides]
public = false
```

and each incoming page will be forced to be private (until you make it public manually, for example with `guestbook review`)

## modular configuration
`guestbook.rs` just works, but can also be deeply customized

by default all supported sql drivers are included, but it's possible to only compile your desired driver by enabling only that feature (`sqlite`, `postgres`, `mysql`)

all notifiers are bundled, but just like db drivers it's possible to only include some (`email`, `telegram`)

the integrated frontend is not necessary, and can be excluded by disabling the `web` feature. serve your favorite page and just use the `/api` endpoint for fetching pages.

### making your own frontend
the JS file used by the builtin frontend is available [here](https://cdn.alemi.dev/guestbook/0.3.1.js):
```
https://cdn.alemi.dev/guestbook/0.3.1.js
```

this provides a plain JS (+jsdoc! check type hints) module exporting one function to hook the automatic infinite scroll fetcher

you can also manage things with your framework of choice, just `GET /api?offset=x&limit=y` to receive a list of public pages
```js
class GuestbookPage {
    id: number
    author: string
    contact: string | undefined
    url: string | undefined
    avatar: string
    body: string,
    date: Date,
}
```

inserting a new page can be done with either a `POST` (+form) or a `PUT` (+json) request on `/api` endpoint
```js
class GuestbookInsertion {
    body: string,
    author: string | undefined,
    contact: string | undefined,
    public: boolean | undefined,
    date: Date | undefined,
}
```

# future work
due to the modular structure of notifiers, i plan to add many more notifiers for various services, such as
 * a cool picture or logo
 * matrix
 * mastodon/pleroma
 * discord
 * ??? open to ideas (:
