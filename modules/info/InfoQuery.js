const Query = require("../types/Query");

class InfoQuery extends Query {
    constructor(video, identifier) {
        super(video.environment, identifier);
        this.video = video;
        this.environment = video.environment;
        this.identifier = identifier;
    }

    async connect() {
        try {
            let args = ["-J", "--flat-playlist"]
            if(this.environment.settings.fileAccessRetries) {
                args.push('--file-access-retries');
                args.push(this.environment.settings.fileAccessRetries);
            }
            if(this.environment.settings.allowUnplayable) {
                args.push('--allow-unplayable-formats');
            }
            this.video.headers.forEach((h) => args.push("--add-headers", h.k + ": " + h.v));
            let data = await this.environment.metadataLimiter.schedule(() => this.start(this.video, args));
            return JSON.parse(data);
        } catch (e) {
            this.environment.errorHandler.checkError(e.stderr, this.identifier)
            return null;
        }
    }
}
module.exports = InfoQuery;
