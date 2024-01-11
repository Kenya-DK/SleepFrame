using MouseKeyboardLibrary;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Timers;
using System.Windows.Forms;
using static SleepFrame.Helper;

namespace SleepFrame.Macros
{
    public class MacroBase
    {
        private System.Timers.Timer _timer;
        private TimeSpan _nextInternal = TimeSpan.Zero;
        private TimeSpan _current = TimeSpan.Zero;
        private TimeSpan _internal;
        private bool _isRunning = false;
        public MacroBase(TimeSpan internalTime)
        {
            _internal = internalTime;
            _timer = new System.Timers. Timer(1000);
            _timer.Elapsed += Timer_Elapsed;
        }

        private void Timer_Elapsed(object sender, ElapsedEventArgs e)
        {
            Console.WriteLine(_current.TotalSeconds);
            if (!_isRunning && (_nextInternal == TimeSpan.Zero || _nextInternal <= _current))
            {
                _isRunning = true;
                Process bProcess = Helper.GetWarframeProcess();
                if (bProcess == null)
                    throw new Exception("Warframe is not running");

                Rect NotepadRect = new Rect();
                GetWindowRect(bProcess.MainWindowHandle, ref NotepadRect);
                var windownRect = NotepadRect.GetCenter();


                Process activeProcess = Helper.GetActiveProcess();
                var cPos = Cursor.Position;

                Helper.SetProcessToForeground(bProcess);
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));

                Cursor.Position = new System.Drawing.Point(GetRandomDelay(windownRect.X-100, windownRect.X + 100), GetRandomDelay(windownRect.Y - 100, windownRect.Y + 100));
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));
                MouseSimulator.Click(MouseButtons.Left);
                //Helper.BlockInput(true);
                Run();
                //Helper.BlockInput(false);
                System.Threading.Thread.Sleep(GetRandomDelay(250, 750));
                Cursor.Position = cPos;
                Helper.SetProcessToForeground(activeProcess);
                _isRunning = false;
                _nextInternal = _nextInternal.Add(_internal);
            }
            _current = _current.Add(new TimeSpan(0, 0, 1));
        }

        public TimeSpan Internal
        {
            get { return _internal; }
            set { _internal = value; }
        }

        public virtual void Run()
        {
            throw new NotImplementedException();
        }

        public virtual void Start()
        {
            _timer.Start();
        }
        public virtual void Stop()
        {
            _timer.Dispose();
        }
    }
}
