const Query = require("../types/Query");
const fs = require('fs');
const regcooki = new RegExp(/([^=]+)=([^;^,]+)(; expires=[^;]+)(; path=([^\;]+))*; domain=([^\;]+)(; Secure)*(; HttpOnly)*; priority=[^;^,]+(; SameSite=([^\,]+))*,?/gm);
            
class InfoQuery extends Query {
    constructor(url, headers, rheaders, identifier, environment) {
        super(environment, identifier);
        this.url = url;
        this.headers = headers;
        this.environment = environment;
        this.identifier = identifier;

        let autocookie='# Netscape HTTP Cookie File\n\n';
        rheaders.forEach(h => {
            if (h.k.toLowerCase() == "set-cookie") {
                const placeholders = h.v.matchAll(regcooki);
                for(const match of placeholders) {
                    if(match == null) continue;
                    if(match[0] == null || match[6] == null) continue;
                    let domain=(match[6][0]=='.'?'https://www.'+match[6].substring(1,match[6].length):match[6]);
                    autocookie=autocookie+"*"+domain+"\tFALSE\t"+match[5]+"\t"+  (match[8]?'TRUE':'FALSE')+"\t"+'0'+"\t"+match[1]+"\t"+match[2]+"\n";
                }
            }
        });

        //autocookie
        this.autocookie = autocookie;
    }

    async connect() {
        try {
            let args = ["-J", "--flat-playlist"]
            if(this.environment.settings.fileAccessRetries) {
                args.push('--file-access-retries');
                args.push(this.environment.settings.fileAccessRetries);
            }
            //serialize auto cookie just before calling ydl
            fs.writeFileSync("autocookie.txt", this.autocookie, "utf8");
            args.push('--cookies');
            args.push('autocookie.txt');

            this.headers.forEach((h) => args.push("--add-headers", h.k + ":" + h.v));

            let data = await this.environment.metadataLimiter.schedule(() => this.start(this.url, args));
            return JSON.parse(data);
        } catch (e) {
            this.environment.errorHandler.checkError(e.stderr, this.identifier)
            return null;
        }
    }
}
module.exports = InfoQuery;
