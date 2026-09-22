// One in-flight report; rapid changes retain only the latest application identity.
export class ForegroundReporter {
    constructor(send) { this.send=send;this.pending=null;this.busy=false;this.stopped=false;this.last=undefined; }
    report(id, force=false) {
        if (this.stopped || (!force && id===this.last)) return;
        this.last=id;this.pending=id;this.flush();
    }
    async flush() {
        if (this.busy || this.stopped) return;
        this.busy=true;
        try {
            while (this.pending!==null && !this.stopped) {
                const id=this.pending;this.pending=null;
                try {await this.send(id);} catch (_) {this.last=undefined;}
            }
        } finally {this.busy=false;}
    }
    stop() {this.stopped=true;this.pending=null;}
}
