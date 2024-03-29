﻿using AutoHotkey.Interop;
using MouseKeyboardLibrary;
using Newtonsoft.Json;
using System;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.Remoting.Messaging;
using System.Timers;
using System.Windows.Forms;
using static SleepFrame.Helper;

namespace SleepFrame.Macros
{
    public class MacroBase
    {
        private string _id;
        private string _settingsPath = "settings";
        private AutoHotkeyEngine _ahk = AutoHotkeyEngine.Instance;
        private readonly System.Timers.Timer _timer = new System.Timers.Timer(1000);
        private TimeSpan _nextInternal = TimeSpan.Zero;
        private TimeSpan _current = TimeSpan.Zero;
        private TimeSpan _notifyTime = TimeSpan.Zero;
        private TimeSpan _internal;
        private bool _isRunning = false;
        private bool _isPaused = false;

        public event EventHandler<string> OnProcess;

        public event EventHandler<string> OnStart;

        public event EventHandler<string> OnStop;

        public event EventHandler<string> OnUpdateStatus;

        public event EventHandler<Tuple<string, string, int>> OnNotify;

        public MacroBase(string id, TimeSpan internalTime)
        {
            _id = id;
            _internal = internalTime;
            _timer.Elapsed += Timer_Elapsed;
        }

        private void Timer_Elapsed(object sender, ElapsedEventArgs e)
        {
            if (_isPaused)
                return;
            if (!_isRunning && (_current == TimeSpan.Zero || _nextInternal <= _current))
            {
                _isRunning = true;
                Process bProcess = GetWarframeProcess() ?? throw new Exception("Warframe is not running");
                Rect NotepadRect = new Rect();
                GetWindowRect(bProcess.MainWindowHandle, ref NotepadRect);
                var (X, Y) = NotepadRect.GetCenter();

                Process activeProcess = Helper.GetActiveProcess();
                var cPos = Cursor.Position;

                Helper.SetProcessToForeground(bProcess);
                System.Threading.Thread.Sleep(GetRandomDelay(50, 100));

                // Set The Cursor Position on Warframe
                Cursor.Position = new System.Drawing.Point(GetRandomDelay(X - 100, X + 100), GetRandomDelay(Y - 100, Y + 100));
                System.Threading.Thread.Sleep(GetRandomDelay(250, 500));

                MouseSimulator.Click(MouseButtons.Left);
                _ahk.ExecRaw($"BlockInput ON");
                System.Threading.Thread.Sleep(GetRandomDelay(50, 100));
                Run();
                System.Threading.Thread.Sleep(GetRandomDelay(50, 100));
                Cursor.Position = cPos;
                SetProcessToForeground(activeProcess);
                _ahk.ExecRaw($"BlockInput OFF");
                _nextInternal = _nextInternal.Add(_internal);

                _isRunning = false;
            }

            if (_isRunning)
                return;
            _current = _current.Add(new TimeSpan(0, 0, 1));

            if (_nextInternal.Subtract(_current) == _notifyTime)
                OnNotify?.Invoke(this, new Tuple<string, string, int>("SleepFrame", "Next run in " + _nextInternal.Subtract(_current).ToString(), 250));

            OnProcess?.Invoke(this, $"Next run in {_nextInternal.Subtract(_current)}");
        }

        /// <summary>
        /// Gets the internal TimeSpan value.
        /// </summary>
        public TimeSpan Internal
        {
            get { return _internal; }
        }

        /// <summary>
        /// Runs the macro. This method should be overridden in a derived class.
        /// </summary>
        public virtual void Run()
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Initializes the macro.
        /// This method is virtual and does not perform any operations by default.
        /// It should be overridden in derived classes to perform any necessary initialization before the macro is used.
        /// </summary>
        public virtual void Initialize()
        {
        }

        /// <summary>
        /// Uninitializes the macro.
        /// This method is virtual and does not perform any operations by default.
        /// It should be overridden in derived classes to perform any necessary operations when the macro is no longer needed.
        /// </summary>
        public virtual void UnInitialize()
        {
        }

        public virtual void LoadSettings()
        {
        }

        public T GetSettings<T>() where T : class
        {
            if (!Directory.Exists(_settingsPath))
                Directory.CreateDirectory(_settingsPath);
            if (!File.Exists(Path.Combine(_settingsPath, _id + ".json")))
                return null;
            return JsonConvert.DeserializeObject<T>(File.ReadAllText(Path.Combine(_settingsPath, _id + ".json")));
        }

        public virtual void SaveSettings<T>(T settings)
        {
            File.WriteAllText(Path.Combine(_settingsPath, _id + ".json"), JsonConvert.SerializeObject(settings));
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

        /// <summary>
        /// Pauses the macro execution by setting the '_isPaused' flag to true.
        /// </summary>
        public void Pause()
        {
            _isPaused = true;
        }

        /// <summary>
        /// Resumes the macro execution by setting the '_isPaused' flag to false.
        /// </summary>
        public void Resume()
        {
            _isPaused = false;
        }

        /// <summary>
        /// Updates the status and triggers the OnUpdateStatus event.
        /// </summary>
        /// <param name="status">The new status to set.</param>
        public void UpdateStatus(string status)
        {
            OnUpdateStatus?.Invoke(this, status);
        }

        #region Method Get Set
        /// <summary>
        /// Gets or sets the messages to send.
        /// </summary>
        /// <remarks>
        ///  This is the message that will be sent to the chat.
        /// </remarks>
        public TimeSpan NotifyTimer
        {
            get { return _notifyTime; }
            set { _notifyTime = value; }
        }

        public AutoHotkeyEngine Ahk
        {
            get { return _ahk; }
            set { _ahk = value; }
        }
        #endregion

    }
}
