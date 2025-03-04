const InfoQuery = require('./InfoQuery');
const Video = require('../types/Video');
const Utils = require("../Utils");

class InfoQueryList {
    constructor(query, headers, environment, progressBar) {
        this.query = query;
        this.environment = environment;
        this.progressBar = progressBar;
        this.urls = null;
        this.length = null;
        this.done = 0;
        this.headers = headers;
    }

    async start() {
        return await new Promise(((resolve) => {
            let totalMetadata = [];
            let playlistUrls = Utils.extractPlaylistUrls(this.query);
            for (const videoData of playlistUrls[1]) {
                let video = this.createVideo(videoData, videoData.url, videoData.headers);
                totalMetadata.push(video);
            }
            this.urls = playlistUrls[0];
            this.length = this.urls.length;
            if (this.length === 0) resolve(totalMetadata);
            if (this.urls === []) {
                console.error("This playlist is empty.");
                this.length = 0;
                resolve(null);
            }
            for (const url of this.urls) {
                let metadataVideo = new Video(url.url, url.headers, "", this.environment);
                let task = new InfoQuery(metadataVideo, "");
                task.connect().then((data) => {
                    if (data.formats != null) {
                        let video = this.createVideo(data, url.url, url.headers);
                        totalMetadata.push(video);
                    }
                    this.done++;
                    this.progressBar.updatePlaylist(this.done, this.length);
                    if (this.done === this.length) {
                        resolve(totalMetadata);
                    }
                });
            }
        }));
    }

    createVideo(data, url, headers) {
        let metadata = (data.entries != null && data.entries.length === 1 && data.entries[0].formats != null) ? data.entries[0] : data;
        let video = new Video(url, headers,"single", this.environment);
        video.setMetadata(metadata);
        return video;
    }

}

module.exports = InfoQueryList;
