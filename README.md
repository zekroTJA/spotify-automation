# Spotify Automations

A small Vercel application to automate some stuff in Spotify.

Currently, this app can do the following stuff.
- Create and update playlists containing your recently most played songs *(variants for short, medium and long term)* [`/api/auto/mostplayed`].
- Archive the contents of your "Discover Weekly" playlist in a separate archival playlist [`/api/auto/dwa`].
- Create and update playlists containing saved tracks released in a given time range.

## Setup

To deploy the Vercel serverless function, you first need to install the [Vercel CLI](https://vercel.com/docs/cli) and log in with your account.

Then, create a [new Project on Vercel](https://vercel.com/new). Also, you need to create a [KV store](https://vercel.com/dashboard/stores) bound to the created project.

Secondly, you need to create a Spotify OAuth application in your [Spotify developer dashboard](https://developer.spotify.com/dashboard). There you will be able to obtain the Client ID and sercet required for later configuration. Also, you need to specify the redirect URL. It is composed of the vercel application URL and the oauth callback route. An example callback URL would look as following.
```
https://my-spotify-automation.vercel.app/api/oauth/callback
```

> For local development, add the following redirect URL.
> ```
> http://localhost:3000/api/oauth/callback
> ```

After that, clone the repository and deploy the app to Vercel afterwards.

```bash
git clone https://github.com/zekroTJA/spotify-automation.git
cd spotify-automation
vercel --prod
```

Now, configure the project via environment variables using the Vercel CLI.

First, the Spotify client ID.
```bash
echo "5U7Eq2tg5YhBThrIi8sKUJ0pVvxlyEtd" \
    | vercel env add SPOTIFY_CLIENTID production
```

Then, the Spotify client secret.
```bash
echo "dhfXfoKnvX4dMWsFg4H0jphs1XFqs2e4" \
    | vercel env add SPOTIFY_CLIENTSECRET production
```

After that, we need to specify the public OAuth2 redirect URL. This must be the same as configured in the Spotify OAuth application.
```bash
echo "https://my-spotify-automation.vercel.app/api/oauth/callback" \
    | vercel env add REDIRECT_URL production
```

Finally, you might need to re-deploy the production application to apply the environment variables to the nevironment.

When everything is set up correctly, you should be able to navigate to the `/api/oauth/login` endpoint and authorize with your Spotify account. This requests a refresh authorization token which is then stored in the Vercel KV database. After that, calling the endpoint `/api/auto/mostplayed` will create a Playlist with the name `Current Top Songs` containing your latest most played songs which is automatically updated every day by a CRON-job.

## Limitations

This project makes use of [Vercel cron jobs](https://vercel.com/docs/cron-jobs), which are currently in beta. In the free tier, you are only able to create a maximum of 2 cron jobs. Also, [according to the documentation](https://vercel.com/docs/cron-jobs#are-cron-jobs-free), cron jobs are only free during the beta phase.

But heads up, you have some options to bypass these limitations.

If you have an own server (Home Server, RaspberryPi, VPS, ...), simply create the cron jobs there calling the public API. Here a small example.
```
0 1 * * TUE curl -L -X GET 'https://my-spotify-automations.vercel.app/api/auto/dwa?dw_name=Discover%20Weekly&dwa_name=Discover%20Weekly%20Archive'
```

Alternatively, you can also use automation services like [IFTTT](https://ifttt.com/) or [Make](https://make.com/). [Here](contrib/make/blueprint.json), you can find an example template for the latter.

## Ideas

Some Ideas which could be implemented into this project in the future if I am bored again.

- [x] Add query params to `/api/auto/mostplayed` like `playlist_name` or `timespan`
- [x] Add automation to store all songs in the "Discover Weekly" playlist into one large archival playlist
- [ ] Add proper handling when the created playlist is deleted