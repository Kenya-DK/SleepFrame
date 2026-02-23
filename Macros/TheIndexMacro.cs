using MouseKeyboardLibrary;
using System;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame.Macros
{
    public class TheIndexMacro : MacroBase
    {
        #region Private Values
        private TimeSpan _lastOpraterMode = TimeSpan.Zero;
        private Task _protectiveSling;
        private Task _opRaterMode;
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

            // Go to Oprater Mode
            // If last oprater mode was more than Random(10,15) seconds ago
            if (DateTime.Now.TimeOfDay - _lastOpraterMode > TimeSpan.FromSeconds(10 + rnd.Next(0, 5)))
            {
                _lastOpraterMode = DateTime.Now.TimeOfDay;
                OpraterMode();
                Thread.Sleep(500 + rnd.Next(0, 300));
                OpraterMode();
            }
            // Move mouse to screen coordinates
            _protectiveSling = Task.Run(ProtectiveSling);
        }
        public override void Stop()
        {
            base.Stop();
            if (_protectiveSling != null)
            {
                _protectiveSling.Wait();
                _protectiveSling.Dispose();
            }
            if (_opRaterMode != null)
            {
                _opRaterMode.Wait();
                _opRaterMode.Dispose();
            }
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
