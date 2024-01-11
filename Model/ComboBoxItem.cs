using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace SleepFrame.Model
{
    public class ComboBoxItem
    {

        /// <summary>
        /// ComobBox object.
        /// </summary>
        public object Vaule { get; set; }

        /// <summary>
        /// ComobBox Item.
        /// </summary>
        public string Text { get; set; }

        /// <summary>
        /// Item image.
        /// </summary>
        public Image Image { get; set; }

        /// <summary>
        /// ComobBox Tag.
        /// </summary>
        public object Tag { get; set; }
        public ComboBoxItem()
        {
            Text = String.Empty;
            Image = new Bitmap(1, 1);
        }


        /// <summary>
        /// Constructor item without image.
        /// </summary>
        /// <param name="value">Item value.</param>
        public ComboBoxItem(string Text)
        {
            this.Text = Text;
            Image = new Bitmap(1, 1);
        }


        /// <summary>
        ///  Constructor item with image.
        /// </summary>
        /// <param name="value">Item value.</param>
        /// <param name="image">Item image.</param>
        public ComboBoxItem(string text, Image image, object value, object tag = null)
        {
            Text = text;
            Image = image;
            Vaule = value;
            Tag = tag;
        }

        public override string ToString()
        {
            return Text.ToString();
        }
    }
}
