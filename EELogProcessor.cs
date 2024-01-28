using System;
using System.Collections.Generic;
using System.IO.MemoryMappedFiles;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.ComTypes;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Text.RegularExpressions;

namespace SleepFrame
{
    public  class EELogEvent
    {
        private string _category;
        private Dictionary<string, Action<Match>> _patterns;


        public EELogEvent(string category, Dictionary<string, Action<Match>> patterns)
        {
            Category = category;
            Patterns = patterns;
        }

        public string Category
        {
            get { return _category; }
            set { _category = value; }
        }


        public Dictionary<string, Action<Match>> Patterns
        {
            get { return _patterns; }
            set { _patterns = value; }
        }

    }
    public static class EELogProcessor
    {
        private static bool _isRunning = false;
        private static List<EELogEvent> _events = new List<EELogEvent>();



        public static void Start()
        {
            _isRunning = true;
            new Thread(() =>
            {
                while (_isRunning)
                {
                    try
                    {
                        using (MemoryMappedFile orOpen = MemoryMappedFile.CreateOrOpen("DBWIN_BUFFER", 4096L))
                        {
                            bool createdNew1;
                            using (EventWaitHandle eventWaitHandle1 = new EventWaitHandle(false, EventResetMode.AutoReset, "DBWIN_BUFFER_READY", out createdNew1))
                            {
                                try
                                {
                                    if (Helper.lastWFProcessID == -1)
                                        continue;

                                    using (EventWaitHandle eventWaitHandle2 = new EventWaitHandle(false, EventResetMode.AutoReset, "DBWIN_DATA_READY", out bool createdNew2))
                                    {
                                        char[] chArray = new char[5000];
                                        while (Helper.lastWFProcessID != -1)
                                        {
                                            eventWaitHandle1.Set();
                                            if (eventWaitHandle2.WaitOne(TimeSpan.FromSeconds(3.0)))
                                                using (MemoryMappedViewStream viewStream = orOpen.CreateViewStream())
                                                using (BinaryReader binaryReader = new BinaryReader(viewStream, Encoding.UTF8))
                                                    if (binaryReader.ReadUInt32() == Helper.lastWFProcessID)
                                                    {
                                                        char[] array = binaryReader.ReadChars(4092);
                                                        ProcessLine(new string(array, 0, Array.IndexOf<char>(array, char.MinValue)));
                                                    }
                                        }
                                    }

                                }
                                catch
                                {
                                    Thread.Sleep(10000);
                                }
                                finally
                                {
                                    eventWaitHandle1.Set();
                                }
                            }
                        }
                    }
                    catch
                    {
                    }
                    finally
                    {
                        Thread.Sleep(10000);
                    }
                }
            })
            {
                IsBackground = true
            }.Start();
        }

        public static void Stop()
        {
            _isRunning = false;
        }

        public static void ProcessLine(string line)
        {
            // Check if any of the events match the line
            Dictionary<string, Action<Match>> patterns = _events.SelectMany(x => x.Patterns).ToDictionary(x => x.Key, x => x.Value);
            foreach (KeyValuePair<string, Action<Match>> pattern in patterns)
            {
                Match match = Regex.Match(line, pattern.Key);
                if (match.Success)
                    pattern.Value(match);
            }
        }


        public static void AddEvent(EELogEvent e)
        {
            _events.Add(e);
        }

        public static void RemoveEventByCategory(string category)
        {
            _events.RemoveAll(x => x.Category == category);
        }

        public static void RemoveEvent(EELogEvent e)
        {
            _events.Remove(e);
        }

    }




}
