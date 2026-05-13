using MouseKeyboardLibrary;
using System;
using System.Threading;
using System.Windows.Forms;

namespace SleepFrame.Macros
{
    public class TheIndexMacro : MacroBase
    {
        #region Private Values
        private TimeSpan _lastOpraterMode = TimeSpan.Zero;
        private CancellationTokenSource _cts;
        #endregion

        #region Events

        #endregion

        #region New
        /// <summary>
        /// Creates a new generic <see cref="TheIndex"/>
        /// </summary>
        public TheIndexMacro() : base("theIndex", new TimeSpan(0, 0, 0, 4, 3000), new TimeSpan(0, 0, 0))
        {
            RandomMaxInternal = 4000;
            RandomMinInternal = 0;
            SetFocus = true;
        }

        #endregion

        #region Events
        private void OnMessagesChanged(object sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
        {
            SaveToFile();
        }
        #endregion

        #region Virtual Override
        public override UserControl GetView()
        {
            return null;
        }
        /// <summary>
        /// Override The Run Method to execute your own code
        /// </summary>
        public override void Run()
        {
            Random rnd = new Random();

            while (!_cts.Token.IsCancellationRequested)
            {
                // Go to Oprater Mode every 15-20 seconds
                if (DateTime.Now.TimeOfDay - _lastOpraterMode > TimeSpan.FromSeconds(15 + rnd.Next(0, 5)))
                {
                    _lastOpraterMode = DateTime.Now.TimeOfDay;
                    OpraterMode();
                    Thread.Sleep(500 + rnd.Next(0, 300));
                    OpraterMode();
                }

                // Run ProtectiveSling then wait 3-4.5 seconds before running again
                ProtectiveSling();
                _cts.Token.WaitHandle.WaitOne(2500 + rnd.Next(0, 1500));
            }
        }
        public override void Start()
        {
            _cts = new CancellationTokenSource();
            base.Start();
        }
        public override void Stop()
        {
            _cts?.Cancel();
            base.Stop();
        }
        #endregion

        #region Method
        private void ProtectiveSling()
        {
            Random rnd = new Random();
            // Send Left control key down
            Ahk.ExecRaw("Send {LControl down}");
            Thread.Sleep(305 + rnd.Next(100, 500));
            // Sends Space key down
            Ahk.ExecRaw("Send {Space down}");
            Thread.Sleep(608 + rnd.Next(100, 500));
            // Sends Space key up
            Ahk.ExecRaw("Send {Space up}");
            // Send Left control key up
            Ahk.ExecRaw("Send {LControl up}");
        }
        private void OpraterMode()
        {
            Random rnd = new Random();
            Ahk.ExecRaw("Send {5 down}");
            Thread.Sleep(50 + rnd.Next(20, 80));
            Ahk.ExecRaw("Send {5 up}");
        }
        #endregion
        #region Method Get Set

        #endregion
    }
}
