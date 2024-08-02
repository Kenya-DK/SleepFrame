using AutoHotkey.Interop;
using MouseKeyboardLibrary;
using Newtonsoft.Json;
using SleepFrame.Properties;
using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Runtime.Remoting.Messaging;
using System.Threading;
using System.Timers;
using System.Windows.Forms;
using static SleepFrame.Helper;

namespace SleepFrame.Macros
{
    public class MacroBase
    {
        #region Private Values
        private string _id;
        private string _settingsPath = "settings";
        private AutoHotkeyEngine _ahk = AutoHotkeyEngine.Instance;
        private readonly System.Timers.Timer _timer = new System.Timers.Timer(1000);
        private TimeSpan _nextInternal = TimeSpan.Zero;
        private TimeSpan _current = TimeSpan.Zero;
        private TimeSpan _internal;
        private TimeSpan _notifyTime = new TimeSpan(0, 0, 5);
        private bool _isRunning = false;
        private bool _isPaused = false;
        #endregion

        #region Events
        public event EventHandler<string> OnProcess;

        public event EventHandler<string> OnStart;

        public event EventHandler<string> OnStop;

        public event EventHandler<string> OnUpdateStatus;

        public event EventHandler<Tuple<string, string, int>> OnNotify;
        #endregion

        #region New
        /// <summary>
        /// Creates a new generic <see cref="MacroBase"/>
        /// </summary>

        public MacroBase(string id, TimeSpan internalTime, TimeSpan notifyTime)
        {
            _id = id;
            _internal = internalTime;
            _timer.Elapsed += Timer_Elapsed;
            _notifyTime = notifyTime;
        }
        #endregion

        #region Method

        private void SetWarframeInFocus()
        {
            // Get the Warframe process
            Process bProcess = GetWarframeProcess() ?? throw new Exception("Warframe is not running");

            // Get the Warframe window rect and center
            Rect NotepadRect = GetWarframeWindow();
            var (X, Y) = NotepadRect.GetCenter();

            // Set the Warframe process to the foreground
            SetProcessToForeground(bProcess);
            Thread.Sleep(GetRandomDelay(50, 100));

            // Set The Cursor Position on Warframe
            Cursor.Position = new System.Drawing.Point(GetRandomDelay(X - 100, X + 100), GetRandomDelay(Y - 100, Y + 100));

            // Wait for the cursor to move
            Thread.Sleep(GetRandomDelay(250, 500));

            // Click on the Warframe window
            MouseSimulator.Click(MouseButtons.Left);
        }

        private void Timer_Elapsed(object sender, ElapsedEventArgs e)
        {
            if (_isPaused)
                return;
            if (!_isRunning && (_current == TimeSpan.Zero || _nextInternal <= _current))
            {
                _isRunning = true;
                SetWarframeInFocus();

                Process activeProcess = GetActiveProcess();
                var cPos = Cursor.Position;

                MouseSimulator.Click(MouseButtons.Left);
                _ahk.ExecRaw($"BlockInput ON");
                Thread.Sleep(GetRandomDelay(50, 100));
                Run();
                Thread.Sleep(GetRandomDelay(50, 100));
                // Restore the cursor position and the active process
                Cursor.Position = cPos;
                SetProcessToForeground(activeProcess);
                _ahk.ExecRaw($"BlockInput OFF");
                _nextInternal = _nextInternal.Add(_internal);

                _isRunning = false;
            }

            if (_isRunning)
                return;
            _current = _current.Add(new TimeSpan(0, 0, 1));

            if (_nextInternal.Subtract(_current) == _notifyTime && _notifyTime > TimeSpan.Zero)
                OnNotify?.Invoke(this, new Tuple<string, string, int>("SleepFrame", "Next run in " + _nextInternal.Subtract(_current).ToString(), 250));

            OnProcess?.Invoke(this, $"Next run in {_nextInternal.Subtract(_current)}");
        }
        #endregion

        #region Virtual Override
        /// <summary>
        /// Runs the macro. This method should be overridden in a derived class.
        /// </summary>
        public virtual void Run()
        {
            throw new NotImplementedException();
        }
        public virtual void SaveToFile()
        {
            if (!Directory.Exists(_settingsPath))
                Directory.CreateDirectory(_settingsPath);
            string json = JsonConvert.SerializeObject(this, Formatting.Indented);
            File.WriteAllText(Path.Combine(_settingsPath, _id + ".json"), json);
        }
        public virtual string LoadFromFile()
        {
            string path = Path.Combine(_settingsPath, _id + ".json");
            if (!Directory.Exists(_settingsPath))
                Directory.CreateDirectory(_settingsPath);
            if (!File.Exists(path))
                return null;
            string json = File.ReadAllText(path);
            MacroBase macro = JsonConvert.DeserializeObject<MacroBase>(json);
            _notifyTime = macro.NotifyTime;
            return path;
        }
        /// <summary>
        /// Gets the view for the macro. This method should be overridden in a derived class.
        /// </summary>
        /// <returns>A UserControl representing the view for the macro.</returns>
        public virtual UserControl GetView()
        {
            throw new NotImplementedException();
        }
        /// <summary>
        /// Starts the macro. This method triggers the OnProcess and OnStart events and starts the timer.
        /// </summary>
        public virtual void Start()
        {
            OnProcess?.Invoke(this, "Starting...");
            OnStart?.Invoke(this, "");
            _timer.Start();
        }
        /// <summary>
        /// Stops the execution of the macro.
        /// This method stops the timer, resets the 'nextInternal' and 'current' timespans to zero,
        /// and invokes the 'OnStop' and 'OnProcess' events with appropriate parameters.
        /// </summary>
        public virtual void Stop()
        {
            _timer.Stop();
            _nextInternal = TimeSpan.Zero;
            _current = TimeSpan.Zero;
            OnStop?.Invoke(this, "");
            OnProcess?.Invoke(this, "Idle");
        }
        #endregion

        #region Method Get Set
        /// <summary>
        /// Gets the internal TimeSpan value.
        /// </summary>
        [JsonIgnore]
        public TimeSpan Internal
        {
            get { return _internal; }
        }

        /// <summary>
        /// Gets the next internal TimeSpan value.
        /// </summary>
        /// <value>The next internal.</value>
        [JsonIgnore]
        public TimeSpan NextInternal
        {
            get { return _nextInternal; }
        }

        /// <summary>
        /// Gets the current TimeSpan value.
        /// </summary>
        [JsonIgnore]
        public TimeSpan Current
        {
            get { return _current; }
        }

        /// <summary>
        /// Gets the AHK value.
        /// </summary>
        [JsonIgnore]
        public AutoHotkeyEngine Ahk
        {
            get { return _ahk; }
            set { _ahk = value; }
        }

        /// <summary>
        /// Gets the notify time TimeSpan value.
        /// </summary>
        [JsonProperty("notify_time")]
        public TimeSpan NotifyTime
        {
            get { return _notifyTime; }
            set { _notifyTime = value; }
        }
        #endregion
    }
}
