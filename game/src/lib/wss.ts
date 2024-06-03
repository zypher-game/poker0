import mitt from 'mitt';

export const ModeCode = {
  MSG: 'message', //普通消息
  HEART_BEAT: 'heart_beat', //心跳
};

const defaultHeartBeat = {
  /**
   * 断线重连时间间隔
   */
  reconnect: 5000,
};

export class Z4WebSocket {
  url: string;
  bus = mitt();

  wss!: WebSocket;

  timeReconnect = 5000;

  lastActiveTime = Date.now();

  closed = false;

  isReconnect = false;

  reconnectTimer: any = null;

  constructor(url: string) {
    this.url = url;
    this.init();
  }

  init() {
    this.wss = new WebSocket(this.url);
    this.wss.onopen = this.onopen.bind(this);
    this.wss.onclose = this.onclose.bind(this);
    this.wss.onmessage = this.onmessage.bind(this);
    this.wss.onerror = this.onerror.bind(this);
  }

  onopen(ev: Event) {
    clearInterval(this.reconnectTimer);
    this.bus.emit(this.isReconnect ? 'reconnect' : 'open', ev);
  }

  onmessage(ev: MessageEvent<any>) {
    // console.log('message', ev.data);
    this.lastActiveTime = Date.now();
    if (ev.data === 'pong') return;
    const data = JSON.parse(ev.data);
    this.bus.emit('message', data);
  }

  onclose(ev: CloseEvent) {
    this.bus.emit('close', ev);
    if (this.closed) return;
    this.reconnect();
  }

  onerror(ev: Event) {
    this.bus.emit('error', ev);
  }

  send(obj: any) {
    const isOpen = this.wss.readyState === this.wss.OPEN;
    const sendMsg = () => {
      this.wss.send(JSON.stringify(obj));
      if (!isOpen) this.bus.off('reconnect', sendMsg);
    };
    if (isOpen) return sendMsg();
    this.bus.on('reconnect', sendMsg);
  }

  close() {
    this.closed = true;
    this.wss.close();
  }

  reconnect() {
    if (this.closed) return;
    console.log('reconnect');
    this.isReconnect = true;
    clearInterval(this.reconnectTimer);
    this.reconnectTimer = setInterval(() => {
      if (this.closed) {
        clearInterval(this.reconnectTimer);
        return;
      }
      this.init();
    }, this.timeReconnect);
  }
}
