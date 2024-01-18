using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace SleepFrame
{
    
    public class Helper
    {
        private enum ShowWindowCommands : int
        {
            Hide = 0,
            Normal = 1,
            Minimized = 2,
            Maximized = 3,
        }

        [DllImport("user32.dll")]
        private static extern int SetForegroundWindow(IntPtr hwnd);
        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool IsIconic(IntPtr hWnd);

        [DllImport("user32.dll")]
        private static extern bool ShowWindowAsync(IntPtr hWnd, ShowWindowCommands cmdShow);

        [DllImport("user32.dll")]
        public static extern IntPtr GetWindowThreadProcessId(IntPtr hWnd, out uint ProcessId);

        [DllImport("user32.dll")]
        private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")]
        public static extern bool GetWindowRect(IntPtr hwnd, ref Rect rectangle);

        /// Return Type: BOOL->int
        ///fBlockIt: BOOL->int
        [System.Runtime.InteropServices.DllImportAttribute("user32.dll", EntryPoint = "BlockInput")]
        [return: System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.Bool)]
        public static extern bool BlockInput([System.Runtime.InteropServices.MarshalAsAttribute(System.Runtime.InteropServices.UnmanagedType.Bool)] bool fBlockIt);



        public struct Rect
        {
            public int Left { get; set; }
            public int Top { get; set; }
            public int Right { get; set; }
            public int Bottom { get; set; }
            // Method to get the center of the rectangle
            public (int X, int Y) GetCenter()
            {
                int centerX = (Left + Right) / 2;
                int centerY = (Top + Bottom) / 2;

                return (centerX, centerY);
            }
        }


        public static Process GetWarframeProcess()
        {
            //return Process.GetProcessesByName("Notepad").FirstOrDefault();
            return Process.GetProcessesByName("Warframe.x64").FirstOrDefault();
        }

        public static Process GetActiveProcess()
        {
            IntPtr hwnd = GetForegroundWindow();
            GetWindowThreadProcessId(hwnd, out uint pid);
            return Process.GetProcessById((int)pid);
        }

        public static void SetProcessToForeground(Process process)
        {
            if (process == null)
                return;
            // check if the process is running
            if (process != null)
            {
                if (IsIconic(process.MainWindowHandle))
                    ShowWindowAsync(process.MainWindowHandle, ShowWindowCommands.Normal);

                // set user the focus to the window
                SetForegroundWindow(process.MainWindowHandle);
            }
        }
        [STAThread]
        public static void Ctrl_V(string text)
        {
            // Get the clipboard object
            //IDataObject clipboardObject = Clipboard.GetDataObject();

            //// Walk through all (relevant) clipboard formats and save them in a new object
            //var formats = clipboardObject.GetFormats(false);
            //Dictionary<string, object> clipboardFormats = new Dictionary<string, object>();
            //foreach (var format in formats)
            //{
            //    if (
            //        format.Contains("Text") ||
            //        format.Contains("Hyperlink") ||
            //        format.Contains("HTML Format") ||
            //        format.Contains("Bitmap")
            //    )
            //    {
            //        // Add the clipboard format to the clipboard object
            //        clipboardFormats.Add(format, clipboardObject.GetData(format));
            //    }
            //}
            // Put new content to the clipboard - just to show we can destroy it before restoring again
            
            Clipboard.SetText(text);
            System.Windows.Forms.SendKeys.SendWait("^{v}");


            //// ---------
            //// Restore the (semi) original clipboard - at least the relevant formats we have saved

            //DataObject data = new DataObject();
            //foreach (KeyValuePair<string, object> kvp in clipboardFormats)            
            //    if (kvp.Value != null)                
            //        data.SetData(kvp.Key, kvp.Value);

            //// Copy the saved formats to the clipboard
            //Clipboard.SetDataObject(data, true);


        }
        public static int GetRandomDelay(int min, int max)
        {
            Random random = new Random();
            return random.Next(min, max);
        }
        public static class Clipboard
        {
            public static void SetText(string p_Text)
            {
                Thread STAThread = new Thread(
                    delegate ()
                    {
                        // Use a fully qualified name for Clipboard otherwise it
                        // will end up calling itself.
                        System.Windows.Forms.Clipboard.SetText(p_Text);
                    });
                STAThread.SetApartmentState(ApartmentState.STA);
                STAThread.Start();
                STAThread.Join();
            }

            public static string GetText()
            {
                string ReturnValue = string.Empty;
                Thread STAThread = new Thread(
                    delegate ()
                    {
                        // Use a fully qualified name for Clipboard otherwise it
                        // will end up calling itself.
                        ReturnValue = System.Windows.Forms.Clipboard.GetText();
                    });
                STAThread.SetApartmentState(ApartmentState.STA);
                STAThread.Start();
                STAThread.Join();

                return ReturnValue;
            }
        }
    }
}
