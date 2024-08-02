using AutoHotkey.Interop;
using Newtonsoft.Json.Linq;
using SleepFrame.Macros;
using SleepFrame.Model;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Drawing.Printing;
using System.Globalization;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using static SleepFrame.Helper;
using static System.Net.Mime.MediaTypeNames;
using static System.Windows.Forms.VisualStyles.VisualStyleElement;

namespace SleepFrame
{
    public partial class MainForm : Form
    {
        Hotkey hkToggle = new Hotkey();
        MacroBase currentMacro;
        bool is_runnig = false;

        public MainForm()
        {
            InitializeComponent();
            LoadProgram();
        }
        private void LoadProgram()
        {
            Event(false);
            _chMacros.Items.Add(new ComboBoxItem("Trading", null, new TradingMacro()));
            //_chMacros.Items.Add(new ComboBoxItem<TheIndex>("The Index", null, new TheIndex()));
            //_chMacros.Items.Add(new ComboBoxItem<NightwaveSkip>("Nightwave Skip", null, new NightwaveSkip()));
            cbKeyToo.DataSource = Enum.GetValues(typeof(MouseShortcut));
            cbKeyOn.DataSource = Enum.GetValues(typeof(MouseShortcut2));

            #region Load AutoClicker Hotkey
            hkToggle.KeyCode = Keys.F3;
            hkToggle.Pressed += delegate
            {
                Toggle();
            };
            hkToggle.Register(this);
            #endregion
            #region Default
            cbKeyOn.SelectedItem = MouseShortcut2.None;
            cbKeyToo.SelectedItem = MouseShortcut.F3;
            _chMacros.SelectedIndex = 0;
            #endregion
            Event(true);
        }

        private void Event(bool Enable)
        {
            if (Enable)
            {
                cbKeyOn.SelectedIndexChanged += Value_Changed;
                cbKeyToo.SelectedIndexChanged += Value_Changed;
            }
            else
            {
                cbKeyOn.SelectedIndexChanged -= Value_Changed;
                cbKeyToo.SelectedIndexChanged -= Value_Changed;
            }
        }
        private void Value_Changed(object sender, EventArgs e)
        {

            hkToggle.KeyCode = GetKeyCodeFromMouseShortcut((MouseShortcut)cbKeyToo.SelectedItem);

            switch ((MouseShortcut2)Enum.Parse(typeof(MouseShortcut2), cbKeyOn.SelectedItem.ToString()))
            {
                case MouseShortcut2.Alt:
                    hkToggle.Alt = true;
                    break;
                case MouseShortcut2.Ctrl:
                    hkToggle.Control = true;
                    break;
                case MouseShortcut2.None:
                    hkToggle.Control = false;
                    break;
                case MouseShortcut2.Shift:
                    hkToggle.Shift = true;
                    break;
            }
        }
        private Keys GetKeyCodeFromMouseShortcut(MouseShortcut Key, MouseShortcut2 Key2 = MouseShortcut2.None)
        {
            if (Key2 == MouseShortcut2.None)
                switch (Key)
                {
                    case MouseShortcut.Ctrl:
                        return Keys.Control;
                    case MouseShortcut.Enter:
                        return Keys.Enter;
                    case MouseShortcut.F1:
                        return Keys.F1;
                    case MouseShortcut.F2:
                        return Keys.F2;
                    case MouseShortcut.F3:
                        return Keys.F3;
                    case MouseShortcut.F4:
                        return Keys.F4;
                    case MouseShortcut.F5:
                        return Keys.F5;
                    case MouseShortcut.F6:
                        return Keys.F6;
                    case MouseShortcut.F7:
                        return Keys.F7;
                    case MouseShortcut.F8:
                        return Keys.F8;
                    case MouseShortcut.F9:
                        return Keys.F9;
                    case MouseShortcut.F10:
                        return Keys.F10;
                    case MouseShortcut.F11:
                        return Keys.F11;
                    case MouseShortcut.F12:
                        return Keys.F12;
                    case MouseShortcut.Shift:
                        return Keys.Shift;
                    default:
                        return Keys.None;
                }
            else
                switch (Key2)
                {
                    case MouseShortcut2.Ctrl:
                        return Keys.Control;
                    case MouseShortcut2.Shift:
                        return Keys.Shift;
                    default:
                        return Keys.None;
                }
        }

        private void _chMacros_SelectedIndexChanged(object sender, EventArgs e)
        {

            ComboBoxItem boxItem = (ComboBoxItem)_chMacros.SelectedItem;

            //if (currentMacro != null)
            //    currentMacro.UnInitialize();

            currentMacro = (MacroBase)boxItem.Vaule;
            currentMacro.LoadFromFile();

            _chNotifyEnable.CheckedChanged -= _chNotifyEnable_CheckedChanged;
            _numNotifyTimer.ValueChanged -= _numNotifyTimer_ValueChanged;
            _chNotifyEnable.Checked = currentMacro.NotifyTime > TimeSpan.Zero;
            _numNotifyTimer.Enabled = _chNotifyEnable.Checked;
            _numNotifyTimer.Value = (int)currentMacro.NotifyTime.TotalSeconds;
            _chNotifyEnable.CheckedChanged += _chNotifyEnable_CheckedChanged;
            _numNotifyTimer.ValueChanged += _numNotifyTimer_ValueChanged;

            currentMacro.OnProcess += CurrentMacro_OnProcess;
            currentMacro.OnStart += CurrentMacro_OnStart;
            currentMacro.OnStop += CurrentMacro_OnStop;
            currentMacro.OnNotify += CurrentMacro_OnNotify;
            currentMacro.OnUpdateStatus += CurrentMacro_OnUpdateStatus;

            groupBox1.Controls.Clear();
            var view = currentMacro.GetView();
            if (currentMacro != null && view != null)
                groupBox1.Controls.Add(view);
        }

        private void CurrentMacro_OnUpdateStatus(object sender, string e)
        {
            _lblTimer.InvokeIfRequired(() => { _lblTimer.Text = "Start"; });
        }

        private void CurrentMacro_OnNotify(object sender, Tuple<string, string, int> e)
        {
            _niApp.BalloonTipTitle = e.Item1;
            _niApp.BalloonTipText = e.Item2;
            _niApp.ShowBalloonTip(e.Item3);
        }

        private void CurrentMacro_OnStop(object sender, string e)
        {
            is_runnig = false;
            _niApp.Icon = Properties.Resources.app_icon;
            _btnStartOrStop.InvokeIfRequired(() => { _btnStartOrStop.Text = "Start"; });
            _niApp.Text = "SleepFrame";
            _niApp.BalloonTipText = "Stopping";
            _niApp.BalloonTipTitle = "Macro Stopped";
            _niApp.ShowBalloonTip(1000);
            _chMacros.InvokeIfRequired(() => { _chMacros.Enabled = true; });
        }

        private void CurrentMacro_OnStart(object sender, string e)
        {
            is_runnig = true;
            _niApp.Icon = Properties.Resources.app_icon_on;
            _btnStartOrStop.InvokeIfRequired(() => { _btnStartOrStop.Text = "Stop"; });
            _chMacros.Enabled = false;
            _niApp.BalloonTipText = "Starting";
            _niApp.BalloonTipTitle = "Macro Started";
            _niApp.ShowBalloonTip(1000);
            _chMacros.InvokeIfRequired(() => { _chMacros.Enabled = false; });
        }

        private void CurrentMacro_OnProcess(object sender, string e)
        {
            _niApp.Text = "SleepFrame - " + e;
            _lblTimer.InvokeIfRequired(() => { _lblTimer.Text = e; });
        }

        public void Toggle()
        {
            if (currentMacro == null) return;
            if (!is_runnig)
                currentMacro.Start();
            else
                currentMacro.Stop();
        }
        #region Event

        private void _btnStartOrStop_Click(object sender, EventArgs e)
        {
            Toggle();
        }
        #endregion

        private void _numNotifyTimer_ValueChanged(object sender, EventArgs e)
        {
            if (currentMacro == null) return;
            currentMacro.NotifyTime= new TimeSpan(0, 0, (int)_numNotifyTimer.Value);
            currentMacro.SaveToFile();
        }

        private void _chNotifyEnable_CheckedChanged(object sender, EventArgs e)
        {
            if (currentMacro == null) return;
            _numNotifyTimer.Enabled = _chNotifyEnable.Checked;
            currentMacro.NotifyTime = _chNotifyEnable.Checked ? new TimeSpan(0, 0, (int)_numNotifyTimer.Value) : TimeSpan.Zero;
            currentMacro.SaveToFile();
        }
    }
    #region Enum
    public enum MouseSpeed { Instant, SuperSlow, Slow, Natural = 3, Fast = 5, SuperFast = 8 };
    public enum MouseMessage { Start = 0, Stop = 1, Errow = 3 };

    public enum MouseClickVaules { Move = 0x0001, Leftdown = 0x0002, Leftup = 0x0004, Rightdown = 0x0008, Rightup = 0x0010, Middledown = 0x0020, Middleup = 0x0040, Xdown = 0x0080, Xup = 0x0100, Wheel = 0x0800, Absolute = 0x8000, }
    public enum MouseShortcut2 { None = -1, Ctrl = 13, Shift = 14, Alt = 15 }
    public enum MouseShortcut { None = -1, F1 = 0, F2 = 1, F3 = 2, F4 = 3, F5 = 4, F6 = 5, F7 = 6, F8 = 7, F9 = 8, F10 = 9, F11 = 10, F12 = 11, Enter = 12, Ctrl = 13, Shift = 14 }
    #endregion

}
