using AutoHotkey.Interop;
using MouseKeyboardLibrary;
using System;
using System.Diagnostics;
using System.Timers;
using System.Windows.Forms;
using static SleepFrame.Helper;

namespace SleepFrame.Macros
{
    public class MacroBase
    {
        private AutoHotkeyEngine _ahk = AutoHotkeyEngine.Instance;
        private readonly System.Timers.Timer _timer = new System.Timers.Timer(1000);
        private TimeSpan _nextInternal = TimeSpan.Zero;
        private TimeSpan _current = TimeSpan.Zero;
        private TimeSpan _notifyTime = TimeSpan.Zero;
        private TimeSpan _internal;
        private bool _isRunning = false;

        public event EventHandler<string> OnProcess;

        public event EventHandler<string> OnStart;

        public event EventHandler<string> OnStop;

        public event EventHandler<Tuple<string, string, int>> OnNotify;

        public MacroBase(TimeSpan notifyTime, TimeSpan internalTime)
        {
            _internal = internalTime;
            _notifyTime = notifyTime;
            _timer.Elapsed += Timer_Elapsed;
        }

        private void Timer_Elapsed(object sender, ElapsedEventArgs e)
        {
            if (!_isRunning && (_current == TimeSpan.Zero || _nextInternal <= _current))
            {
                _isRunning = true;
                Process bProcess = Helper.GetWarframeProcess();
                if (bProcess == null)
                    throw new Exception("Warframe is not running");

                Rect NotepadRect = new Rect();
                GetWindowRect(bProcess.MainWindowHandle, ref NotepadRect);
                var (X, Y) = NotepadRect.GetCenter();

                Process activeProcess = Helper.GetActiveProcess();
                var cPos = Cursor.Position;

                Helper.SetProcessToForeground(bProcess);
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));

                Cursor.Position = new System.Drawing.Point(GetRandomDelay(X - 100, X + 100), GetRandomDelay(Y - 100, Y + 100));
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));
                MouseSimulator.Click(MouseButtons.Left);
                //Helper.BlockInput(true);
                System.Threading.Thread.Sleep(GetRandomDelay(750, 850));
                Run();
                Helper.BlockInput(false);
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));
                Cursor.Position = cPos;
                Helper.SetProcessToForeground(activeProcess);
                _nextInternal = _nextInternal.Add(_internal);

                _isRunning = false;
            }

            if (_isRunning)
                return;
            _current = _current.Add(new TimeSpan(0, 0, 1));

            Console.WriteLine(_nextInternal.Subtract(_current) == _notifyTime);
            Console.WriteLine(_nextInternal);
            Console.WriteLine(_current);
            if (_nextInternal.Subtract(_current) == _notifyTime)            
                OnNotify?.Invoke(this, new Tuple<string, string, int>("SleepFrame", "Next run in " + _nextInternal.Subtract(_current).ToString(), 250));

            OnProcess?.Invoke(this, $"Next run in {_nextInternal.Subtract(_current)}");
        }

        public TimeSpan Internal
        {
            get { return _internal; }
        }

        public virtual void Run()
        {
            throw new NotImplementedException();
        }

        public virtual void Start()
        {
            OnProcess?.Invoke(this, "Starting...");
            OnStart?.Invoke(this, "");
            _timer.Start();
        }
        public virtual void Stop()
        {
            _timer.Stop();
            _nextInternal = TimeSpan.Zero;
            _current = TimeSpan.Zero;
            OnStop?.Invoke(this, "");
            OnProcess?.Invoke(this, "Idle");
        }

        public AutoHotkeyEngine Ahk
        {
            get { return _ahk; }
            set { _ahk = value; }
        }

    }
}
