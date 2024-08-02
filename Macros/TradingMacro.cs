using AutoHotkey.Interop;
using MouseKeyboardLibrary;
using Newtonsoft.Json;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using static SleepFrame.Helper;
using System.Timers;
using System.Windows.Forms;
using System.Collections.ObjectModel;
using System.Runtime.Remoting.Messaging;

namespace SleepFrame.Macros
{
    public class TradingMacro : MacroBase
    {
        #region Private Values
        private ObservableCollection<TradingMessage> _messages = new ObservableCollection<TradingMessage>();
        private int _index = 0;
        #endregion

        #region Events

        #endregion

        #region New
        /// <summary>
        /// Creates a new generic <see cref="Trading"/>
        /// </summary>
        public TradingMacro() : base("trading", new TimeSpan(0, 2, 0), new TimeSpan(0, 0, 5))
        {
            _messages.CollectionChanged += OnMessagesChanged;
        }

        #endregion

        #region Events
        private void OnMessagesChanged(object sender, System.Collections.Specialized.NotifyCollectionChangedEventArgs e)
        {
            SaveToFile();
        }
        #endregion
        #region Method

        /// <summary>
        /// Add a message to the list
        /// </summary>
        public void AddMessage(TradingMessage message)
        {
            _messages.Add(message);
        }

        /// <summary>
        /// Remove a message from the list
        /// </summary>
        public void RemoveMessage(TradingMessage message)
        {
            _messages.Remove(message);
        }
        #endregion

        #region Virtual Override
        public override UserControl GetView()
        {
            return new Views.TradingUserControl(this);
        }
        public override string LoadFromFile()
        {
            string path = base.LoadFromFile();
            if (path == null)
                return null;
            string json = File.ReadAllText(path);
            TradingMacro macro = JsonConvert.DeserializeObject<TradingMacro>(json);
            _messages = macro._messages;
            return path;
        }
        /// <summary>
        /// Override The Run Method to execute your own code
        /// </summary>
        public override void Run()
        {
            List<string> messages = _messages.Where(x => x.Enable).Select(x => x.Message).ToList();
            if (messages.Count == 0)
                return;
            if (_index >= messages.Count)
                _index = 0;

            // Sends the message
            Ahk.ExecRaw($"SendInput {messages[_index]}");
            Thread.Sleep(GetRandomDelay(50, 100));
            // Sends the enter key
            Ahk.ExecRaw("Send {enter}");
            Thread.Sleep(GetRandomDelay(50, 100));
            // Sends the t key to open the chat again
            Ahk.ExecRaw("Send t");
            // Removes the t typed when not in dojo 
            Ahk.ExecRaw("Send {BackSpace}");
            Thread.Sleep(GetRandomDelay(50, 100));

            _index++;
            if (_index >= messages.Count)
                _index = 0;
        }
        #endregion
        #region Method Get Set
        /// <summary>
        /// Get the list of messages
        /// </summary>
        [JsonProperty("messages")]
        public ObservableCollection<TradingMessage> Messages
        {
            get { return _messages; }
        }
        /// <summary>
        /// Get the index
        /// </summary>
        [JsonIgnore]
        public int Index
        {
            get { return _index; }
            set { _index = value; }
        }
        #endregion
    }
    public class TradingMessage
    {
        #region Private Values
        private string _message;
        private bool _enable;
        #endregion

        #region Events

        #endregion

        #region New
        /// <summary>
        /// Creates a new generic <see cref="TradingMessage"/>
        /// </summary>
        public TradingMessage(string message, bool enable)
        {
            _message = message;
            _enable = enable;
        }
        #endregion

        #region Method
        #endregion

        #region Virtual Override

        #endregion
        #region Method Get Set
        /// <summary>
        /// Get the message
        /// </summary>
        [JsonProperty("message")]
        public string Message
        {
            get { return _message; }
            set { _message = value; }
        }
        /// <summary>
        /// Get the enable
        /// </summary>
        [JsonProperty("enable")]
        public bool Enable
        {
            get { return _enable; }
            set { _enable = value; }
        }
        #endregion
    }
}
