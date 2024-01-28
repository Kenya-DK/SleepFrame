using SleepFrame.Macros.Views;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace SleepFrame.Macros
{
    /// <summary>
    /// Trading.
    /// </summary>
    public class Trading : MacroBase
    {

        #region Const/Static Values
        #endregion
        #region Private Values		

        #endregion
        #region New

        /// <summary>
        /// Creates a new generic <see cref="Trading"/>
        /// </summary>
        public Trading() : base(new TimeSpan(0, 0, 35), new TimeSpan(0, 2, 0))
        {
            _messages = new List<string>();
        }
        /// <summary>
        /// Creates a new generic <see cref="Trading"/>
        /// </summary>
        public Trading(List<string> messages) : this()
        {
            _messages = messages;
        }
        /// <summary>
        /// Creates a new generic <see cref="Trading"/>
        /// </summary>
        public Trading(string msg) : this()
        {
            _messages = new List<string>() { msg };
        }

        #endregion
        #region Method 			
        private List<string> _messages;
        private int _index = 0;
        #endregion
        #region Override Method      
        /// <summary>
        /// Override ToString for <see cref="Trading"/>
        /// </summary>
        /// <returns>
        /// Returns store data in string.
        /// </returns>
        public override string ToString()
        {
            return base.ToString();
        }

        /// <summary>
        /// Override The Run Method to execute your own code
        /// </summary>
        public override void Run()
        {

            if (_messages.Count == 0)
                return;
            if (_index >= _messages.Count)
                _index = 0;

            // Sends the message
            Ahk.ExecRaw($"SendInput {_messages[_index]}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            // Sends the enter key
            Ahk.ExecRaw("Send {enter}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            // Sends the t key to open the chat again
            Ahk.ExecRaw("Send t");
            // Removes the t typed when not in dojo 
            Ahk.ExecRaw("Send {BackSpace}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));

            _index++;
            if (_index >= _messages.Count)
                _index = 0;
        }

        public override UserControl GetView()
        {
            return new Views.TradingUserControl(this);
        }
        #endregion
        #region Method Get Set
        /// <summary>
        /// Gets or sets the messages to send.
        /// </summary>
        /// <remarks>
        ///  This is the message that will be sent to the chat.
        /// </remarks>
        public List<string> Messages
        {
            get { return _messages; }
            set { _messages = value; }
        }

        /// <summary>
        /// Gets or sets the index of the message to send.
        /// </summary>
        /// <remarks>
        /// This is the index of the message that will be sent to the chat.
        /// </remarks>
        /// <exception cref="ArgumentOutOfRangeException">
        /// Thrown when the index is out of range.
        /// </exception>
        public int Index
        {
            get { return _index; }
            set
            {
                if (value < 0 || value >= _messages.Count)
                    throw new ArgumentOutOfRangeException("Index is out of range.");
                _index = value;
            }
        }
        #endregion


        #region EELogs Methods 

        public void AddEeLogEvents()
        {
            EELogProcessor.AddEvent(new EELogEvent("tradechat", new Dictionary<string, Action<System.Text.RegularExpressions.Match>>()
                {
                    { @"E\S{9}: L", Loading },
                    { @"E\S{9}: S", Synchronizing },
                    { @"E\S{9}: C", Connected },
                })
            );
        }

        public void RemoveEeLogEvents()
        {
            EELogProcessor.RemoveEventByCategory("tradechat");
        }

        private void Loading(Match match)
        {
            Console.WriteLine("Loading");
        }

        private void Synchronizing(Match match)
        {
            Console.WriteLine("Synchronizing");
        }


        private void Connected(Match match)
        {
            Console.WriteLine("Connected");
        }

        #endregion

    }
}