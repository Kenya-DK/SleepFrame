using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
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
        public Trading(string msg) : base(new TimeSpan(0, 0, 35), new TimeSpan(0, 2, 0))
        {
            _message = msg;
        }

        #endregion
        #region Method 			
        private string _message;
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
            // Removes the t typed when not in dojo 
            Ahk.ExecRaw("Send {BackSpace}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            // Sends the message
            Ahk.ExecRaw($"SendInput {_message}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            // Sends the enter key
            Ahk.ExecRaw("Send {enter}");
            System.Threading.Thread.Sleep(Helper.GetRandomDelay(50, 100));
            // Sends the t key to open the chat again
            Ahk.ExecRaw("Send t");
        }

        #endregion
        #region Method Get Set

        /// <summary>
        /// Gets or sets the message to send.
        /// </summary>
        /// <remarks>
        ///  This is the message that will be sent to the chat.
        /// </remarks>
        public string Message
        {
            get { return _message; }
            set { _message = value; }
        }
        #endregion
    }
}
