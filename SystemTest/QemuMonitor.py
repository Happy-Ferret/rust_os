import select
import subprocess
import threading
import thread

class KillerThread:
    def __init__(self):
        self._event = threading.Event()
        self._start = threading.Event()
        self._run = True
        self._th = thread.start_new_thread(KillerThread.run, (self,))
    def reset(self):
        self._event.set()
    def start(self):
        self._start.set()
    def kill(self):
        self._run = False
        self._start.set()
    def run(self):
        while self._run:
            #print "- Waiting to time"
            self._start.wait()
            if not self._run:
                break
            #print "- Timing 2s"
            self._start.clear()
            if self._event.wait(2.0) == None:
                thread.interrupt_main()
            #print "- Done"
            self._event.clear()

class QemuMonitor:
    def __init__(self, cmd_strings):
        self._instance = subprocess.Popen(cmd_strings, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self._mode = ''
        self._timer = KillerThread()
    def __del__(self):
        self.cmd("quit")
        while True:
            line = self.get_line(timeout=0.5)
            if line == None:
                break
            print "QemuMonitor.__del__ -", line
        self._instance.terminate()
        self._timer.kill()
        print "Killing qemu instance"
    def send_key(self, keycode):
        self.cmd('sendkey %s' % keycode)
    def send_combo(self, keycodes):
        self.cmd('sendkey %s' % '-'.join(keycodes))
    def mouse_move(self, dx, dy):
        self.cmd('mouse_move %i %i' % (dx,dy))
    def mouse_button(self, mask):
        self.cmd('mouse_button %i' % (mask,))
    
    def get_line(self, timeout=1.0):
        r,_w,_e = select.select( [self._instance.stdout], [], [], timeout)
        if len(r) > 0:
            try:
                self._timer.start()
                s = self._instance.stdout.readline()
            except KeyboardInterrupt:
                print "--- ERROR: Timeout (or SIGINT) during readline()"
                raise
            finally:
                self._timer.reset()
            if s == "":
                return None
            return s.strip()
        else:
            return None
    
    def send_screendump(self, path):
        self.cmd('screendump %s' % (path,))

    def cmd(self, string):
        if self._mode != 'monitor':
            self._instance.stdin.write('\1c')
            self._mode = 'monitor'
            line = self.get_line(timeout=1)
        
        self._instance.stdin.flush()
        print ">> CMD:", string
        self._instance.stdin.write(string)
        self._instance.stdin.write('\n')
        self._instance.stdin.flush()
    
        line = self.get_line(timeout=1)
        #if line != '(qemu) %s' % (string):
        #    print "Unexpected response: '%s', expected '%s'" % (line, '(qemu) %s' % (string)) 
        #    raise "Doop"
        #line = self.get_line(timeout=1)
        #if line != '(qemu)':
        #    print "Unexpected response: %s" % (line,) 
        #    raise "Doop"
        